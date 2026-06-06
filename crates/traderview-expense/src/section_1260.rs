//! IRC § 1260 — Gains From Constructive Ownership
//! Transactions / Pass-Thru Entity Loophole-Closer Module.
//!
//! Pure-compute check for IRC § 1260 ordinary-income
//! recharacterization + interest-charge treatment of gains
//! from constructive-ownership transactions involving
//! pass-thru entity interests. § 1260 was added by Public
//! Law 106-170 § 534(a) (Ticket to Work and Work Incentives
//! Improvement Act of 1999) on December 17, 1999, effective
//! for transactions entered into AFTER JULY 11, 1999. The
//! provision closes the derivative-replication loophole that
//! had allowed hedge funds and other taxpayers to obtain
//! long-term capital gains treatment by using total return
//! swaps, forwards, or coupled call/put structures to
//! synthetically hold pass-thru entity interests instead of
//! buying the underlying interest directly.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1260(a) Treatment of Gain**: if gain from a
//!   constructive-ownership transaction with respect to any
//!   financial asset WOULD be treated as long-term capital
//!   gain, such gain is treated as ORDINARY INCOME to the
//!   extent that it EXCEEDS the **NET UNDERLYING LONG-TERM
//!   CAPITAL GAIN** (NULTCG); the residual amount equal to
//!   NULTCG retains long-term capital gain character
//!   ([Cornell LII 26 USC § 1260](https://www.law.cornell.edu/uscode/text/26/1260);
//!   [Bloomberg Tax Sec. 1260](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1260)).
//! - **IRC § 1260(b) Interest Charge**: in addition to the
//!   ordinary-income recharacterization, an interest charge
//!   under § 6601 is imposed on the ordinary-income portion;
//!   interest is computed assuming the gain accrued at a
//!   **CONSTANT RATE equal to the applicable Federal rate
//!   (AFR)** in effect on the day the transaction CLOSED;
//!   the interest charge cannot be used as a credit against
//!   any other tax liability.
//! - **IRC § 1260(c) Definitions**:
//!   - **Financial Asset**: (1) equity interest in a pass-thru
//!     entity; (2) debt instrument as provided by regulations;
//!     (3) stock in a non-pass-thru corporation as provided by
//!     regulations.
//!   - **Pass-Thru Entity** (eight enumerated types under
//!     § 1260(c)(2)): (i) regulated investment company (RIC);
//!     (ii) real estate investment trust (REIT); (iii) S
//!     corporation; (iv) partnership; (v) trust; (vi) common
//!     trust fund; (vii) passive foreign investment company
//!     (PFIC); (viii) real estate mortgage investment conduit
//!     (REMIC).
//! - **IRC § 1260(d) Constructive Ownership Transaction
//!   Definition**: taxpayer is treated as having entered into
//!   a constructive ownership transaction if the taxpayer:
//!   (1) holds a LONG POSITION UNDER A NOTIONAL PRINCIPAL
//!   CONTRACT with respect to the financial asset;
//!   (2) enters into a FORWARD OR FUTURES CONTRACT to acquire
//!   the financial asset;
//!   (3) is the HOLDER OF A CALL OPTION AND THE GRANTOR OF A
//!   PUT OPTION with respect to the financial asset with
//!   substantially equal strike prices AND substantially
//!   contemporaneous maturity dates (classic "collar around
//!   zero" or synthetic forward construction); OR
//!   (4) enters into ONE OR MORE OTHER TRANSACTIONS THAT HAVE
//!   SUBSTANTIALLY THE SAME EFFECT, as provided by regulations
//!   ([Bloomberg Tax / Mondaq — Constructive Ownership
//!   Transactions](https://www.mondaq.com/unitedstates/corporate-tax/12132/constructive-ownership-transactions)).
//! - **IRC § 1260(d)(1)(A) Long Position Under NPC**: a person
//!   is treated as holding a long position under a notional
//!   principal contract with respect to any financial asset
//!   if such person (i) has the right to be PAID (or receive
//!   credit for) ALL OR SUBSTANTIALLY ALL OF THE INVESTMENT
//!   YIELD (including appreciation) on such financial asset
//!   for a specified period, AND (ii) is OBLIGATED TO
//!   REIMBURSE (or provide credit for) ALL OR SUBSTANTIALLY
//!   ALL OF ANY DECLINE in the value of such financial asset.
//! - **IRC § 1260(c)(2) Mark-to-Market Exception**: § 1260
//!   does NOT apply if ALL of the positions which are part of
//!   the transaction are marked to market under any other
//!   provision of the IRC or its regulations (covers
//!   § 1256 contracts, dealer M2M, § 475 trader M2M election).
//! - **IRC § 1260(e) Net Underlying Long-Term Capital Gain**:
//!   the long-term capital gain that the taxpayer WOULD HAVE
//!   HAD if the financial asset had been (i) ACQUIRED FOR FAIR
//!   MARKET VALUE on the date the constructive ownership
//!   transaction was OPENED, and (ii) SOLD FOR FAIR MARKET
//!   VALUE on the date the transaction was CLOSED; the amount
//!   of net underlying long-term capital gain must be
//!   established by the taxpayer by CLEAR AND CONVINCING
//!   EVIDENCE.
//! - **Effective Date**: § 1260 applies to transactions
//!   ENTERED INTO AFTER JULY 11, 1999; enactment date was
//!   December 17, 1999 (Public Law 106-170 § 534(a)).
//! - **Companion Provisions**: § 1259 (constructive sale of
//!   appreciated financial position — built earlier);
//!   § 1258 (conversion transactions — built iter 654);
//!   § 1092 (straddle rules); § 1256 (60/40 mark-to-market for
//!   listed contracts); § 475(f) (trader mark-to-market
//!   election); § 6601 (interest on underpayments).
//! - **Trader / Hedge-Fund Significance**: § 1260 is the
//!   anti-abuse rule targeting hedge-fund "synthetic equity"
//!   structures that used total-return swaps over RIC, REIT,
//!   or partnership interests to convert what would have been
//!   ordinary pass-thru income (interest, dividends, ordinary
//!   recharacterized items) into long-term capital gain at
//!   the swap close-out date. § 1260 ordinary-income
//!   recharacterization + AFR interest charge eliminates the
//!   tax-arbitrage incentive.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1260_ENACTMENT_YEAR: u32 = 1999;
pub const IRC_1260_ENACTMENT_MONTH: u32 = 12;
pub const IRC_1260_ENACTMENT_DAY: u32 = 17;
pub const IRC_1260_EFFECTIVE_DATE_YEAR: u32 = 1999;
pub const IRC_1260_EFFECTIVE_DATE_MONTH: u32 = 7;
pub const IRC_1260_EFFECTIVE_DATE_DAY: u32 = 11;
pub const IRC_1260_NUMBER_OF_ENUMERATED_PASS_THRU_ENTITY_TYPES: u32 = 8;
pub const IRC_1260_NUMBER_OF_CONSTRUCTIVE_OWNERSHIP_TRANSACTION_TYPES: u32 = 4;
pub const IRC_1260_AFR_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionEntryDateStatus {
    EnteredAfterJuly11_1999EffectiveDate,
    EnteredOnOrBeforeJuly11_1999BeforeEffectiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinancialAssetType {
    PassThruEntityInterestRegulatedInvestmentCompany,
    PassThruEntityInterestRealEstateInvestmentTrust,
    PassThruEntityInterestSCorporation,
    PassThruEntityInterestPartnership,
    PassThruEntityInterestTrust,
    PassThruEntityInterestCommonTrustFund,
    PassThruEntityInterestPassiveForeignInvestmentCompany,
    PassThruEntityInterestRealEstateMortgageInvestmentConduit,
    NotPassThruEntityInterestExcludedFromSection1260,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructiveOwnershipTransactionType {
    LongPositionUnderNotionalPrincipalContractUnderSection1260D1A,
    ForwardOrFuturesContractToAcquireFinancialAssetUnderSection1260D1B,
    CallHolderAndPutGrantorSubstantiallyEqualStrikeAndContemporaneousMaturityUnderSection1260D1C,
    OtherSubstantiallySameEffectTransactionUnderSection1260D1D,
    NotConstructiveOwnershipTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkToMarketStatus {
    AllPositionsMarkedToMarketUnderSection1260C2Exception,
    NotAllPositionsMarkedToMarket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GainStatus {
    GainWouldBeLongTermCapitalGainExceedsNetUnderlyingLongTermCapitalGain,
    GainWouldBeLongTermCapitalGainEqualToOrBelowNetUnderlyingLongTermCapitalGain,
    GainIsShortTermNoSection1260RecharacterizationApplicable,
    NoGainOrLoss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1260Mode {
    NotApplicableTransactionEnteredOnOrBeforeJuly11_1999BeforeEffectiveDate,
    NotApplicableNotPassThruEntityInterest,
    NotApplicableNotConstructiveOwnershipTransaction,
    NotApplicableAllPositionsMarkedToMarketUnderSection1260C2Exception,
    NotApplicableNoGainOrShortTermGain,
    CompliantGainEqualToOrBelowNetUnderlyingLongTermCapitalGainNoOrdinaryRecharacterization,
    ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub transaction_entry_date_status: TransactionEntryDateStatus,
    pub financial_asset_type: FinancialAssetType,
    pub constructive_ownership_transaction_type: ConstructiveOwnershipTransactionType,
    pub mark_to_market_status: MarkToMarketStatus,
    pub gain_status: GainStatus,
    pub total_gain_dollars: u64,
    pub net_underlying_long_term_capital_gain_dollars: u64,
    pub applicable_federal_rate_basis_points_annual: u64,
    pub number_of_full_years_transaction_open: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1260Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub ordinary_income_recharacterization_dollars: u64,
    pub interest_charge_dollars: u64,
}

pub type Section1260Input = Input;
pub type Section1260Output = Output;
pub type Section1260Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1260(a) Treatment of Gain — if gain from constructive ownership transaction with respect to any financial asset WOULD be treated as long-term capital gain, such gain treated as ORDINARY INCOME to extent it EXCEEDS the NET UNDERLYING LONG-TERM CAPITAL GAIN (NULTCG); residual amount equal to NULTCG retains long-term capital gain character".to_string(),
        "IRC § 1260(b) Interest Charge — in addition to ordinary-income recharacterization, interest charge under § 6601 is imposed on ordinary-income portion; interest computed assuming gain accrued at CONSTANT RATE equal to applicable Federal rate (AFR) in effect on day transaction CLOSED, compounded semiannually; interest charge cannot be credited against any other tax".to_string(),
        "IRC § 1260(c)(1) Financial Asset Definition — (1) equity interest in pass-thru entity; (2) debt instrument as provided by regulations; (3) stock in non-pass-thru corporation as provided by regulations".to_string(),
        "IRC § 1260(c)(2) Pass-Thru Entity Eight Enumerated Types — (i) regulated investment company (RIC); (ii) real estate investment trust (REIT); (iii) S corporation; (iv) partnership; (v) trust; (vi) common trust fund; (vii) passive foreign investment company (PFIC); (viii) real estate mortgage investment conduit (REMIC)".to_string(),
        "IRC § 1260(d)(1) Constructive Ownership Transaction Definition — (A) LONG POSITION UNDER NOTIONAL PRINCIPAL CONTRACT; (B) FORWARD OR FUTURES CONTRACT TO ACQUIRE FINANCIAL ASSET; (C) HOLDER OF CALL OPTION AND GRANTOR OF PUT OPTION WITH SUBSTANTIALLY EQUAL STRIKE PRICES AND SUBSTANTIALLY CONTEMPORANEOUS MATURITY DATES; (D) OTHER TRANSACTIONS WITH SUBSTANTIALLY THE SAME EFFECT (regulatory)".to_string(),
        "IRC § 1260(d)(1)(A) Long Position Under NPC — person treated as holding long position if (i) has right to be PAID (or receive credit for) ALL OR SUBSTANTIALLY ALL OF THE INVESTMENT YIELD including appreciation for specified period, AND (ii) OBLIGATED TO REIMBURSE (or provide credit for) ALL OR SUBSTANTIALLY ALL OF ANY DECLINE in value".to_string(),
        "IRC § 1260(c)(2) Mark-to-Market Exception — § 1260 does NOT apply if ALL positions which are part of the transaction are marked to market under any other IRC provision or regulation (covers § 1256 contracts, dealer M2M, § 475(f) trader M2M election)".to_string(),
        "IRC § 1260(e) Net Underlying Long-Term Capital Gain Definition — long-term capital gain taxpayer WOULD HAVE HAD if financial asset had been (i) ACQUIRED FOR FAIR MARKET VALUE on date transaction OPENED, and (ii) SOLD FOR FAIR MARKET VALUE on date transaction CLOSED; amount must be established by CLEAR AND CONVINCING EVIDENCE".to_string(),
        "Effective Date — § 1260 applies to transactions ENTERED INTO AFTER JULY 11, 1999; enactment date December 17, 1999 (Public Law 106-170 § 534(a) — Ticket to Work and Work Incentives Improvement Act of 1999)".to_string(),
        "Companion Provisions — § 1259 (constructive sale of appreciated financial position); § 1258 (conversion transactions); § 1092 (straddle rules); § 1256 (60/40 mark-to-market for listed contracts); § 475(f) (trader mark-to-market election); § 6601 (interest on underpayments)".to_string(),
        "Cornell LII 26 USC § 1260 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 1260 — comprehensive code commentary".to_string(),
        "Mondaq — Constructive Ownership Transactions practitioner analysis".to_string(),
        "Public Law 106-170 § 534(a) — original enactment of § 1260 as part of Ticket to Work and Work Incentives Improvement Act of 1999, signed December 17, 1999".to_string(),
    ];

    if input.transaction_entry_date_status
        == TransactionEntryDateStatus::EnteredOnOrBeforeJuly11_1999BeforeEffectiveDate
    {
        return Output {
            mode: Section1260Mode::NotApplicableTransactionEnteredOnOrBeforeJuly11_1999BeforeEffectiveDate,
            statutory_basis: "Public Law 106-170 § 534(a) effective date — § 1260 applies only to transactions entered into AFTER July 11, 1999".to_string(),
            notes: "NOT APPLICABLE: transaction entered on or before July 11, 1999; pre-effective-date transaction; § 1260 does not apply.".to_string(),
            citations,
            ordinary_income_recharacterization_dollars: 0,
            interest_charge_dollars: 0,
        };
    }

    if input.financial_asset_type
        == FinancialAssetType::NotPassThruEntityInterestExcludedFromSection1260
    {
        return Output {
            mode: Section1260Mode::NotApplicableNotPassThruEntityInterest,
            statutory_basis: "IRC § 1260(c)(1)/(c)(2) — § 1260 applies only to financial assets that are pass-thru entity interests (RIC, REIT, S-corp, partnership, trust, CTF, PFIC, REMIC)".to_string(),
            notes: "NOT APPLICABLE: financial asset is not a pass-thru entity interest within § 1260(c)(2)'s eight enumerated categories; § 1260 ordinary-income recharacterization does not apply.".to_string(),
            citations,
            ordinary_income_recharacterization_dollars: 0,
            interest_charge_dollars: 0,
        };
    }

    if input.constructive_ownership_transaction_type
        == ConstructiveOwnershipTransactionType::NotConstructiveOwnershipTransaction
    {
        return Output {
            mode: Section1260Mode::NotApplicableNotConstructiveOwnershipTransaction,
            statutory_basis: "IRC § 1260(d)(1) — § 1260 applies only to constructive ownership transactions (NPC long, fwd/futures, call+put collar, or substantially-same-effect)".to_string(),
            notes: "NOT APPLICABLE: transaction is not a constructive ownership transaction within § 1260(d)(1)'s four enumerated categories; § 1260 does not apply.".to_string(),
            citations,
            ordinary_income_recharacterization_dollars: 0,
            interest_charge_dollars: 0,
        };
    }

    if input.mark_to_market_status
        == MarkToMarketStatus::AllPositionsMarkedToMarketUnderSection1260C2Exception
    {
        return Output {
            mode: Section1260Mode::NotApplicableAllPositionsMarkedToMarketUnderSection1260C2Exception,
            statutory_basis: "IRC § 1260(c)(2) flush language — § 1260 does NOT apply if ALL positions are marked to market under any other IRC provision".to_string(),
            notes: "NOT APPLICABLE: all positions which are part of the constructive ownership transaction are marked to market under another IRC provision (e.g., § 1256, § 475 trader M2M, dealer M2M); § 1260(c)(2) M2M exception applies; no ordinary-income recharacterization.".to_string(),
            citations,
            ordinary_income_recharacterization_dollars: 0,
            interest_charge_dollars: 0,
        };
    }

    match input.gain_status {
        GainStatus::NoGainOrLoss | GainStatus::GainIsShortTermNoSection1260RecharacterizationApplicable => {
            return Output {
                mode: Section1260Mode::NotApplicableNoGainOrShortTermGain,
                statutory_basis: "IRC § 1260(a) — recharacterization triggered only if gain would be long-term capital gain".to_string(),
                notes: "NOT APPLICABLE: no gain on transaction OR gain is short-term capital gain (not long-term); § 1260(a) ordinary-income recharacterization triggered only for what would otherwise be long-term capital gain.".to_string(),
                citations,
                ordinary_income_recharacterization_dollars: 0,
                interest_charge_dollars: 0,
            };
        }
        GainStatus::GainWouldBeLongTermCapitalGainEqualToOrBelowNetUnderlyingLongTermCapitalGain => {
            return Output {
                mode: Section1260Mode::CompliantGainEqualToOrBelowNetUnderlyingLongTermCapitalGainNoOrdinaryRecharacterization,
                statutory_basis: "IRC § 1260(a)/(e) — recharacterization only to extent gain EXCEEDS net underlying long-term capital gain (NULTCG)".to_string(),
                notes: format!(
                    "COMPLIANT: total gain {} ≤ net underlying long-term capital gain {} (NULTCG); no excess to recharacterize as ordinary income under § 1260(a); entire gain retains long-term capital gain character; no interest charge under § 1260(b).",
                    input.total_gain_dollars,
                    input.net_underlying_long_term_capital_gain_dollars
                ),
                citations,
                ordinary_income_recharacterization_dollars: 0,
                interest_charge_dollars: 0,
            };
        }
        GainStatus::GainWouldBeLongTermCapitalGainExceedsNetUnderlyingLongTermCapitalGain => {}
    }

    let excess_gain_dollars = input
        .total_gain_dollars
        .saturating_sub(input.net_underlying_long_term_capital_gain_dollars);

    let interest_charge_dollars = u128::from(excess_gain_dollars)
        .saturating_mul(u128::from(
            input.applicable_federal_rate_basis_points_annual,
        ))
        .saturating_mul(u128::from(input.number_of_full_years_transaction_open))
        .checked_div(u128::from(IRC_1260_AFR_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;

    Output {
        mode: Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed,
        statutory_basis: "IRC § 1260(a) + (b) — gain exceeds NULTCG; excess recharacterized as ordinary income + AFR interest charge under § 6601".to_string(),
        notes: format!(
            "VIOLATION OF SAFE TREATMENT: total gain {} EXCEEDS net underlying long-term capital gain {} (NULTCG); excess {} recharacterized as ORDINARY INCOME under § 1260(a); AFR interest charge approximation at {} basis points × {} years on excess = {} (linear approximation; statutory § 1260(b) uses semi-annual compounding under § 6601 — final amount must be computed using the actual AFR table for each prior year of accrual).",
            input.total_gain_dollars,
            input.net_underlying_long_term_capital_gain_dollars,
            excess_gain_dollars,
            input.applicable_federal_rate_basis_points_annual,
            input.number_of_full_years_transaction_open,
            interest_charge_dollars
        ),
        citations,
        ordinary_income_recharacterization_dollars: excess_gain_dollars,
        interest_charge_dollars,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            transaction_entry_date_status:
                TransactionEntryDateStatus::EnteredAfterJuly11_1999EffectiveDate,
            financial_asset_type:
                FinancialAssetType::PassThruEntityInterestRegulatedInvestmentCompany,
            constructive_ownership_transaction_type:
                ConstructiveOwnershipTransactionType::LongPositionUnderNotionalPrincipalContractUnderSection1260D1A,
            mark_to_market_status: MarkToMarketStatus::NotAllPositionsMarkedToMarket,
            gain_status:
                GainStatus::GainWouldBeLongTermCapitalGainExceedsNetUnderlyingLongTermCapitalGain,
            total_gain_dollars: 1_000_000,
            net_underlying_long_term_capital_gain_dollars: 400_000,
            applicable_federal_rate_basis_points_annual: 500,
            number_of_full_years_transaction_open: 3,
        }
    }

    #[test]
    fn transaction_before_effective_date_not_applicable() {
        let mut input = baseline_input();
        input.transaction_entry_date_status =
            TransactionEntryDateStatus::EnteredOnOrBeforeJuly11_1999BeforeEffectiveDate;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableTransactionEnteredOnOrBeforeJuly11_1999BeforeEffectiveDate
        );
        assert_eq!(output.ordinary_income_recharacterization_dollars, 0);
        assert_eq!(output.interest_charge_dollars, 0);
    }

    #[test]
    fn non_pass_thru_asset_not_applicable() {
        let mut input = baseline_input();
        input.financial_asset_type =
            FinancialAssetType::NotPassThruEntityInterestExcludedFromSection1260;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableNotPassThruEntityInterest
        );
    }

    #[test]
    fn not_constructive_ownership_transaction_not_applicable() {
        let mut input = baseline_input();
        input.constructive_ownership_transaction_type =
            ConstructiveOwnershipTransactionType::NotConstructiveOwnershipTransaction;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableNotConstructiveOwnershipTransaction
        );
    }

    #[test]
    fn all_positions_marked_to_market_exception_applies() {
        let mut input = baseline_input();
        input.mark_to_market_status =
            MarkToMarketStatus::AllPositionsMarkedToMarketUnderSection1260C2Exception;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableAllPositionsMarkedToMarketUnderSection1260C2Exception
        );
        assert!(output.notes.contains("M2M exception"));
    }

    #[test]
    fn short_term_gain_not_applicable() {
        let mut input = baseline_input();
        input.gain_status = GainStatus::GainIsShortTermNoSection1260RecharacterizationApplicable;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableNoGainOrShortTermGain
        );
    }

    #[test]
    fn no_gain_or_loss_not_applicable() {
        let mut input = baseline_input();
        input.gain_status = GainStatus::NoGainOrLoss;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::NotApplicableNoGainOrShortTermGain
        );
    }

    #[test]
    fn gain_equal_to_nultcg_no_recharacterization() {
        let mut input = baseline_input();
        input.gain_status =
            GainStatus::GainWouldBeLongTermCapitalGainEqualToOrBelowNetUnderlyingLongTermCapitalGain;
        input.total_gain_dollars = 400_000;
        input.net_underlying_long_term_capital_gain_dollars = 400_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::CompliantGainEqualToOrBelowNetUnderlyingLongTermCapitalGainNoOrdinaryRecharacterization
        );
        assert_eq!(output.ordinary_income_recharacterization_dollars, 0);
        assert_eq!(output.interest_charge_dollars, 0);
    }

    #[test]
    fn gain_below_nultcg_no_recharacterization() {
        let mut input = baseline_input();
        input.gain_status =
            GainStatus::GainWouldBeLongTermCapitalGainEqualToOrBelowNetUnderlyingLongTermCapitalGain;
        input.total_gain_dollars = 300_000;
        input.net_underlying_long_term_capital_gain_dollars = 400_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::CompliantGainEqualToOrBelowNetUnderlyingLongTermCapitalGainNoOrdinaryRecharacterization
        );
    }

    #[test]
    fn gain_exceeds_nultcg_npc_long_position_ordinary_recharacterization_plus_interest() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
        assert_eq!(output.ordinary_income_recharacterization_dollars, 600_000);
        // 600_000 × 500 bps × 3 years / 10_000 = 600_000 × 500 × 3 / 10_000 = 90_000
        assert_eq!(output.interest_charge_dollars, 90_000);
    }

    #[test]
    fn gain_exceeds_nultcg_forward_futures_recharacterization() {
        let mut input = baseline_input();
        input.constructive_ownership_transaction_type =
            ConstructiveOwnershipTransactionType::ForwardOrFuturesContractToAcquireFinancialAssetUnderSection1260D1B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
        assert_eq!(output.ordinary_income_recharacterization_dollars, 600_000);
    }

    #[test]
    fn gain_exceeds_nultcg_call_plus_put_collar_recharacterization() {
        let mut input = baseline_input();
        input.constructive_ownership_transaction_type =
            ConstructiveOwnershipTransactionType::CallHolderAndPutGrantorSubstantiallyEqualStrikeAndContemporaneousMaturityUnderSection1260D1C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn gain_exceeds_nultcg_other_substantially_same_effect_recharacterization() {
        let mut input = baseline_input();
        input.constructive_ownership_transaction_type =
            ConstructiveOwnershipTransactionType::OtherSubstantiallySameEffectTransactionUnderSection1260D1D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_reit_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type =
            FinancialAssetType::PassThruEntityInterestRealEstateInvestmentTrust;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_s_corp_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type = FinancialAssetType::PassThruEntityInterestSCorporation;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_partnership_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type = FinancialAssetType::PassThruEntityInterestPartnership;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_trust_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type = FinancialAssetType::PassThruEntityInterestTrust;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_common_trust_fund_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type = FinancialAssetType::PassThruEntityInterestCommonTrustFund;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_pfic_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type =
            FinancialAssetType::PassThruEntityInterestPassiveForeignInvestmentCompany;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn pass_thru_entity_remic_recharacterization() {
        let mut input = baseline_input();
        input.financial_asset_type =
            FinancialAssetType::PassThruEntityInterestRealEstateMortgageInvestmentConduit;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1260Mode::ViolationGainExceedsNetUnderlyingLongTermCapitalGainOrdinaryRecharacterizationAndInterestChargeOwed
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1260_ENACTMENT_YEAR, 1999);
        assert_eq!(IRC_1260_ENACTMENT_MONTH, 12);
        assert_eq!(IRC_1260_ENACTMENT_DAY, 17);
        assert_eq!(IRC_1260_EFFECTIVE_DATE_YEAR, 1999);
        assert_eq!(IRC_1260_EFFECTIVE_DATE_MONTH, 7);
        assert_eq!(IRC_1260_EFFECTIVE_DATE_DAY, 11);
        assert_eq!(IRC_1260_NUMBER_OF_ENUMERATED_PASS_THRU_ENTITY_TYPES, 8);
        assert_eq!(
            IRC_1260_NUMBER_OF_CONSTRUCTIVE_OWNERSHIP_TRANSACTION_TYPES,
            4
        );
        assert_eq!(IRC_1260_AFR_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_section_1260_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1260(a)"));
        assert!(joined.contains("§ 1260(b)"));
        assert!(joined.contains("§ 1260(c)(2)"));
        assert!(joined.contains("§ 1260(d)(1)"));
        assert!(joined.contains("§ 1260(e)"));
        assert!(joined.contains("Public Law 106-170"));
        assert!(joined.contains("JULY 11, 1999"));
        assert!(joined.contains("NET UNDERLYING LONG-TERM CAPITAL GAIN"));
    }

    #[test]
    fn afr_interest_charge_arithmetic_three_years_500_basis_points() {
        let mut input = baseline_input();
        input.total_gain_dollars = 200_000;
        input.net_underlying_long_term_capital_gain_dollars = 100_000;
        input.applicable_federal_rate_basis_points_annual = 500;
        input.number_of_full_years_transaction_open = 3;
        let output = check(&input);
        // Excess 100_000 × 500 bps × 3 years / 10_000 = 100_000 × 1500 / 10_000 = 15_000
        assert_eq!(output.ordinary_income_recharacterization_dollars, 100_000);
        assert_eq!(output.interest_charge_dollars, 15_000);
    }

    #[test]
    fn afr_interest_charge_zero_years_zero_charge() {
        let mut input = baseline_input();
        input.number_of_full_years_transaction_open = 0;
        let output = check(&input);
        assert_eq!(output.ordinary_income_recharacterization_dollars, 600_000);
        assert_eq!(output.interest_charge_dollars, 0);
    }

    #[test]
    fn afr_interest_charge_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.total_gain_dollars = u64::MAX;
        input.net_underlying_long_term_capital_gain_dollars = 0;
        input.applicable_federal_rate_basis_points_annual = u64::MAX;
        input.number_of_full_years_transaction_open = u32::MAX;
        let output = check(&input);
        // Saturating arithmetic prevents panic; result clamped to u64::MAX or zero
        assert_eq!(output.ordinary_income_recharacterization_dollars, u64::MAX);
        // u128 multiplication saturates at u128::MAX, division by 10_000 → very large value clamped to u64::MAX
        assert_eq!(output.interest_charge_dollars, u64::MAX);
    }
}
