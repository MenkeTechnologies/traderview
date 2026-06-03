//! IRC § 1274 — Determination of Issue Price in the Case
//! of Certain Debt Instruments Issued for Property /
//! Applicable Federal Rate (AFR) Three-Tier Framework
//! Module.
//!
//! Pure-compute check for IRC § 1274 issue-price determination for debt instruments issued in exchange for property, and the AFR three-tier framework (short-term term ≤ 3 years; mid-term 3-9 years; long-term over 9 years) that drives § 1274, § 6621(b) federal short-term rate determination, § 1258(d) conversion-transaction applicable rate, § 1260(b) constructive-ownership-transaction interest charge, and § 1286/§ 1273 OID adequate-stated-interest test. § 1274 is the foundational anti-abuse rule that prevents seller-financed property transactions from concealing imputed interest as disguised principal.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1274(a) General Rule for Issue Price**: the
//!   issue price for any debt instrument to which § 1274
//!   applies is (1) the **STATED PRINCIPAL AMOUNT** if there
//!   is **ADEQUATE STATED INTEREST** (i.e., the stated
//!   interest rate equals or exceeds the AFR), OR (2) the
//!   **IMPUTED PRINCIPAL AMOUNT** computed under § 1274(b)
//!   if there is NOT adequate stated interest ([Cornell LII
//!   26 USC § 1274](https://www.law.cornell.edu/uscode/text/26/1274);
//!   [Accounting Insights — What Is Section 1274 and How
//!   Does It Determine Issue Price?](https://accountinginsights.org/what-is-section-1274-and-how-does-it-determine-issue-price/);
//!   [CCH AnswerConnect § 1274 Issue Price of Debt
//!   Instruments Issued for Property](https://answerconnect.cch.com/document/arp1033dbaae07c5710009bfb90b11c18cbab03/federal/irc/explanation/1274-issue-price-of-debt-instruments-issued-for-property-for-oid-rules-based-on-adjusted-federal-rate-afr)).
//! - **IRC § 1274(b) Imputed Principal Amount**: the
//!   imputed principal amount equals the **SUM of the
//!   PRESENT VALUES of ALL PAYMENTS** due under the debt
//!   instrument, computed using the **APPLICABLE FEDERAL
//!   RATE COMPOUNDED SEMIANNUALLY** as of the sale or
//!   exchange date. **EXCEPTION — Potentially Abusive
//!   Situations**: in tax-shelter arrangements or in
//!   transactions involving nonrecourse financing or
//!   unusually long terms, the **FAIR MARKET VALUE** of
//!   the property received governs (i.e., issue price =
//!   FMV of property).
//! - **IRC § 1274(c) Applicability Scope**: § 1274 applies to a debt instrument given in exchange for property if (1) some or all of the payments are due MORE THAN 6 MONTHS AFTER the date of the sale or exchange, AND (2) the debt instrument has total payments exceeding the stated principal amount or imputed principal amount under § 1274(b). Key statutory exceptions under § 1274(c)(3): (A) sales of farms for $1,000,000 OR LESS by individuals, estates, testamentary trusts, or small businesses; (B) sales of principal residences; (C) sales involving total payments of $250,000 OR LESS in the aggregate; (D) publicly traded debt instruments governed by § 1273(b); (E) transfers of patents for contingent royalty amounts under § 1235.
//! - **IRC § 1274(d) Applicable Federal Rate Three-Tier Framework** ([Accounting Insights — How IRC Section 1274(d) Determines the Applicable Federal Rate](https://accountinginsights.org/how-irc-section-1274d-determines-the-applicable-federal-rate/); [LegalClarity — What Is the Applicable Federal Rate Under Section 1274(d)?](https://legalclarity.org/what-is-the-applicable-federal-rate-under-section-1274d/); [IRS Rev. Rul. 2024-12 — Section 1274 AFR](https://www.irs.gov/pub/irs-drop/rr-24-12.pdf); [IRS Rev. Rul. 2025-01 — Section 1274 AFR](https://www.irs.gov/pub/irs-drop/rr-25-01.pdf); [IRS Rev. Rul. 2025-19 — Section 1274 AFR](https://www.irs.gov/pub/irs-drop/rr-25-19.pdf); [IRS Rev. Rul. 2025-21 — Section 1274 AFR](https://www.irs.gov/pub/irs-drop/rr-25-21.pdf); [IRS Rev. Rul. 2026-2 — Section 1274 AFR](https://www.irs.gov/pub/irs-drop/rr-26-02.pdf)): SHORT-TERM AFR applies to debt instruments with term of NOT OVER 3 YEARS; MID-TERM AFR applies to terms OVER 3 YEARS BUT NOT OVER 9 YEARS; LONG-TERM AFR applies to terms OVER 9 YEARS. The AFRs are determined monthly by the IRS based on the average market yield on outstanding Treasury obligations of comparable maturity. Taxpayers may elect to use the LOWEST of the three monthly AFRs from the 3 MONTHS preceding the binding contract date (the three-month lookback rule).
//! - **IRC § 1274(e) Sale-Leaseback Adjustment**: when property is sold and then immediately leased back to the seller (or related party), the AFR used for issue-price determination increases to 110 PERCENT of the applicable Federal rate, compounded semiannually, to prevent abuse of the seller-financed-sale-leaseback structure.
//! - **Cross-Reference to § 483 Unstated Interest**: § 483
//!   imposes parallel imputed-interest rules on cash-basis
//!   contract payments where § 1274 does not apply (e.g.,
//!   farms under $1,000,000, principal residences, and
//!   < $250,000 transactions); § 483 and § 1274 form
//!   complementary anti-abuse regime.
//! - **Cross-Reference to § 1273 OID Determination**: § 1274
//!   issue price feeds the § 1273(a)(1) OID amount =
//!   (stated redemption price at maturity − issue price).
//! - **Trader / Tax-Practitioner Significance**: § 1274 is
//!   the operational spine for every AFR-driven tax
//!   computation in the IRC. Every interest-rate citation
//!   in § 6621(b), § 1258(d), § 1260(b), § 7872(f)(2),
//!   § 6601, § 6611, and §§ 1271-1288 ultimately points
//!   to the § 1274(d) AFR table published monthly by the
//!   IRS.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1274_SHORT_TERM_AFR_MAX_YEARS: u32 = 3;
pub const IRC_1274_MID_TERM_AFR_MAX_YEARS: u32 = 9;
pub const IRC_1274_TOTAL_PAYMENTS_EXCEPTION_THRESHOLD_DOLLARS: u64 = 250_000;
pub const IRC_1274_FARM_SALE_EXCEPTION_THRESHOLD_DOLLARS: u64 = 1_000_000;
pub const IRC_1274_SALE_LEASEBACK_AFR_MULTIPLIER_BASIS_POINTS: u64 = 11_000;
pub const IRC_1274_AFR_LOOKBACK_MONTHS: u32 = 3;
pub const IRC_1274_PAYMENT_TIMING_THRESHOLD_MONTHS: u32 = 6;
pub const IRC_1274_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    DebtInstrumentInExchangeForProperty,
    OtherTransactionNotCoveredBySection1274,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExceptionStatus {
    NoExceptionFullyCoveredBySection1274,
    FarmSaleAtOrBelow1MillionByIndividualOrSmallBusinessSection1274C3A,
    PrincipalResidenceSaleSection1274C3B,
    TotalPaymentsAtOrBelow250000DollarsSection1274C3C,
    PubliclyTradedDebtInstrumentSection1274C3D,
    PatentTransferUnderSection1235Section1274C3E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentTimingStatus {
    AtLeastOnePaymentDueMoreThan6MonthsAfterSale,
    AllPaymentsWithin6MonthsOfSaleSection1274C1RequirementNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtTermCategory {
    ShortTermAfrTermAtOrUnder3Years,
    MidTermAfrTermOver3YearsButAtOrUnder9Years,
    LongTermAfrTermOver9Years,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1274Mode {
    NotApplicableNotDebtInstrumentForProperty,
    NotApplicableAllPaymentsWithin6MonthsOfSale,
    NotApplicableExceptionFarmSaleUnder1MillionSection1274C3A,
    NotApplicableExceptionPrincipalResidenceSection1274C3B,
    NotApplicableExceptionTotalPaymentsAtOrBelow250000Section1274C3C,
    NotApplicableExceptionPubliclyTradedDebtSection1274C3D,
    NotApplicableExceptionPatentTransferSection1274C3E,
    CompliantAdequateStatedInterestIssuePriceEqualsStatedPrincipalSection1274A1,
    CompliantImputedPrincipalAmountUsedAsIssuePriceSection1274A2AndB,
    CompliantSaleLeasebackUses110PctOfAfrSection1274E,
    CompliantPotentiallyAbusiveSituationFmvOfPropertyGovernsSection1274B3,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub exception_status: ExceptionStatus,
    pub payment_timing_status: PaymentTimingStatus,
    pub debt_term_category: DebtTermCategory,
    pub stated_interest_rate_basis_points: u64,
    pub applicable_federal_rate_basis_points: u64,
    pub stated_principal_dollars: u64,
    pub imputed_principal_dollars: u64,
    pub fair_market_value_of_property_dollars: u64,
    pub is_sale_leaseback: bool,
    pub is_potentially_abusive_situation: bool,
    pub total_payments_aggregate_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1274Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub determined_issue_price_dollars: u64,
    pub applicable_rate_basis_points: u64,
}

pub type Section1274Input = Input;
pub type Section1274Output = Output;
pub type Section1274Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1274(a) General Rule for Issue Price — issue price = (1) STATED PRINCIPAL AMOUNT if adequate stated interest (stated interest rate ≥ AFR) OR (2) IMPUTED PRINCIPAL AMOUNT under § 1274(b) if not adequate stated interest".to_string(),
        "IRC § 1274(b) Imputed Principal Amount — sum of PRESENT VALUES of ALL PAYMENTS due under debt instrument, computed using applicable Federal rate COMPOUNDED SEMIANNUALLY as of sale or exchange date".to_string(),
        "IRC § 1274(b)(3) Exception for Potentially Abusive Situations — in tax-shelter arrangements or transactions involving nonrecourse financing or unusually long terms, FAIR MARKET VALUE of property received governs (issue price = FMV of property)".to_string(),
        "IRC § 1274(c) Applicability Scope — applies if (1) some payments due MORE THAN 6 MONTHS after sale AND (2) debt instrument total payments exceed stated principal amount or imputed principal amount".to_string(),
        "IRC § 1274(c)(3) Key Statutory Exceptions — (A) farm sales ≤ $1,000,000 by individuals, estates, testamentary trusts, or small businesses; (B) principal residence sales; (C) total payments ≤ $250,000 aggregate; (D) publicly traded debt instruments under § 1273(b); (E) patent transfers for contingent royalty amounts under § 1235".to_string(),
        "IRC § 1274(d) Applicable Federal Rate Three-Tier Framework — SHORT-TERM AFR: term ≤ 3 years; MID-TERM AFR: term > 3 years but ≤ 9 years; LONG-TERM AFR: term > 9 years; AFRs determined monthly by IRS based on Treasury obligation yields; taxpayer may elect LOWEST of three monthly AFRs from 3 months preceding binding contract date (three-month lookback rule)".to_string(),
        "IRC § 1274(e) Sale-Leaseback Adjustment — when property sold and immediately leased back to seller or related party, AFR used for issue-price determination increases to 110 PERCENT of applicable Federal rate, compounded semiannually".to_string(),
        "Cross-Reference to § 483 Unstated Interest — § 483 imposes parallel imputed-interest rules on cash-basis contract payments where § 1274 does not apply (farms < $1M, principal residences, < $250,000 transactions)".to_string(),
        "Cross-Reference to § 1273 OID Determination — § 1274 issue price feeds § 1273(a)(1) OID amount = (stated redemption price at maturity − issue price)".to_string(),
        "Quarterly Rate Publication — IRS publishes Revenue Ruling each month announcing § 1274(d) AFRs for the upcoming month (e.g., Rev. Rul. 2026-2, 2025-21, 2025-19, 2025-01, 2024-12); cumulative AFR table maintained at IRS.gov".to_string(),
        "Companion Provisions — § 6621(b) federal short-term rate (determined under § 1274(d) AFR methodology); § 1258(d) conversion transaction applicable rate; § 1260(b) constructive ownership transaction AFR interest charge; § 7872(f)(2) below-market loan AFR; § 6601 / § 6611 / § 1271-1288 underpayment / overpayment / OID interest rate inputs".to_string(),
        "Cornell LII 26 USC § 1274 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 1274 — comprehensive code commentary".to_string(),
        "CCH AnswerConnect § 1274 Issue Price of Debt Instruments Issued for Property — practitioner guide".to_string(),
    ];

    if input.transaction_type == TransactionType::OtherTransactionNotCoveredBySection1274 {
        return Output {
            mode: Section1274Mode::NotApplicableNotDebtInstrumentForProperty,
            statutory_basis: "IRC § 1274(c)(1) — applies only to debt instruments given in exchange for property".to_string(),
            notes: "NOT APPLICABLE: transaction is not a debt instrument issued in exchange for property; § 1274 does not apply.".to_string(),
            citations,
            determined_issue_price_dollars: 0,
            applicable_rate_basis_points: 0,
        };
    }

    if input.payment_timing_status
        == PaymentTimingStatus::AllPaymentsWithin6MonthsOfSaleSection1274C1RequirementNotMet
    {
        return Output {
            mode: Section1274Mode::NotApplicableAllPaymentsWithin6MonthsOfSale,
            statutory_basis: "IRC § 1274(c)(1) — applies only if some payments due more than 6 months after sale".to_string(),
            notes: "NOT APPLICABLE: all payments due within 6 months of sale; § 1274(c)(1) scope requirement not met; no imputed-interest analysis required.".to_string(),
            citations,
            determined_issue_price_dollars: input.stated_principal_dollars,
            applicable_rate_basis_points: 0,
        };
    }

    match input.exception_status {
        ExceptionStatus::FarmSaleAtOrBelow1MillionByIndividualOrSmallBusinessSection1274C3A => {
            return Output {
                mode: Section1274Mode::NotApplicableExceptionFarmSaleUnder1MillionSection1274C3A,
                statutory_basis: "IRC § 1274(c)(3)(A) — farm sale ≤ $1,000,000 by individual or small business exception".to_string(),
                notes: "NOT APPLICABLE: farm sale at or below $1,000,000 by individual, estate, testamentary trust, or small business; § 1274(c)(3)(A) exception applies; § 483 unstated-interest rules apply instead.".to_string(),
                citations,
                determined_issue_price_dollars: input.stated_principal_dollars,
                applicable_rate_basis_points: 0,
            };
        }
        ExceptionStatus::PrincipalResidenceSaleSection1274C3B => {
            return Output {
                mode: Section1274Mode::NotApplicableExceptionPrincipalResidenceSection1274C3B,
                statutory_basis: "IRC § 1274(c)(3)(B) — principal residence sale exception".to_string(),
                notes: "NOT APPLICABLE: sale of principal residence; § 1274(c)(3)(B) exception applies; § 483 unstated-interest rules apply instead.".to_string(),
                citations,
                determined_issue_price_dollars: input.stated_principal_dollars,
                applicable_rate_basis_points: 0,
            };
        }
        ExceptionStatus::TotalPaymentsAtOrBelow250000DollarsSection1274C3C => {
            return Output {
                mode: Section1274Mode::NotApplicableExceptionTotalPaymentsAtOrBelow250000Section1274C3C,
                statutory_basis: "IRC § 1274(c)(3)(C) — total payments ≤ $250,000 aggregate exception".to_string(),
                notes: format!(
                    "NOT APPLICABLE: total payments aggregate ${} ≤ $250,000 threshold; § 1274(c)(3)(C) exception applies; § 483 unstated-interest rules apply instead.",
                    input.total_payments_aggregate_dollars
                ),
                citations,
                determined_issue_price_dollars: input.stated_principal_dollars,
                applicable_rate_basis_points: 0,
            };
        }
        ExceptionStatus::PubliclyTradedDebtInstrumentSection1274C3D => {
            return Output {
                mode: Section1274Mode::NotApplicableExceptionPubliclyTradedDebtSection1274C3D,
                statutory_basis: "IRC § 1274(c)(3)(D) — publicly traded debt instrument exception (governed by § 1273(b))".to_string(),
                notes: "NOT APPLICABLE: publicly traded debt instrument; § 1274(c)(3)(D) exception applies; issue price determined under § 1273(b) (market price) instead.".to_string(),
                citations,
                determined_issue_price_dollars: input.stated_principal_dollars,
                applicable_rate_basis_points: 0,
            };
        }
        ExceptionStatus::PatentTransferUnderSection1235Section1274C3E => {
            return Output {
                mode: Section1274Mode::NotApplicableExceptionPatentTransferSection1274C3E,
                statutory_basis: "IRC § 1274(c)(3)(E) — patent transfer for contingent royalty amounts exception under § 1235".to_string(),
                notes: "NOT APPLICABLE: patent transfer for contingent royalty amounts under § 1235; § 1274(c)(3)(E) exception applies.".to_string(),
                citations,
                determined_issue_price_dollars: input.stated_principal_dollars,
                applicable_rate_basis_points: 0,
            };
        }
        ExceptionStatus::NoExceptionFullyCoveredBySection1274 => {}
    }

    if input.is_potentially_abusive_situation {
        return Output {
            mode: Section1274Mode::CompliantPotentiallyAbusiveSituationFmvOfPropertyGovernsSection1274B3,
            statutory_basis: "IRC § 1274(b)(3) — potentially abusive situation; FMV of property governs".to_string(),
            notes: format!(
                "POTENTIALLY ABUSIVE SITUATION: tax-shelter arrangement or nonrecourse financing or unusually long terms identified; § 1274(b)(3) treats FMV of property received (${}) as the issue price; stated principal (${}) and imputed principal (${}) ignored.",
                input.fair_market_value_of_property_dollars,
                input.stated_principal_dollars,
                input.imputed_principal_dollars
            ),
            citations,
            determined_issue_price_dollars: input.fair_market_value_of_property_dollars,
            applicable_rate_basis_points: input.applicable_federal_rate_basis_points,
        };
    }

    if input.is_sale_leaseback {
        let leaseback_afr = u128::from(input.applicable_federal_rate_basis_points)
            .saturating_mul(u128::from(IRC_1274_SALE_LEASEBACK_AFR_MULTIPLIER_BASIS_POINTS))
            .checked_div(u128::from(IRC_1274_BASIS_POINT_DENOMINATOR))
            .unwrap_or(0)
            .min(u128::from(u64::MAX)) as u64;
        return Output {
            mode: Section1274Mode::CompliantSaleLeasebackUses110PctOfAfrSection1274E,
            statutory_basis: "IRC § 1274(e) — sale-leaseback uses 110 % of AFR".to_string(),
            notes: format!(
                "SALE-LEASEBACK: § 1274(e) increases AFR to 110 % of applicable Federal rate; effective rate = {} bps (= AFR {} bps × 110 %); imputed principal recomputation required at the higher rate.",
                leaseback_afr, input.applicable_federal_rate_basis_points
            ),
            citations,
            determined_issue_price_dollars: input.imputed_principal_dollars,
            applicable_rate_basis_points: leaseback_afr,
        };
    }

    if input.stated_interest_rate_basis_points >= input.applicable_federal_rate_basis_points {
        Output {
            mode: Section1274Mode::CompliantAdequateStatedInterestIssuePriceEqualsStatedPrincipalSection1274A1,
            statutory_basis: "IRC § 1274(a)(1) — adequate stated interest; issue price = stated principal amount".to_string(),
            notes: format!(
                "COMPLIANT: stated interest rate {} bps ≥ applicable Federal rate {} bps ({} AFR category); ADEQUATE STATED INTEREST exists; § 1274(a)(1) treats issue price as stated principal amount ${}; no imputed-interest reclassification.",
                input.stated_interest_rate_basis_points,
                input.applicable_federal_rate_basis_points,
                match input.debt_term_category {
                    DebtTermCategory::ShortTermAfrTermAtOrUnder3Years => "short-term",
                    DebtTermCategory::MidTermAfrTermOver3YearsButAtOrUnder9Years => "mid-term",
                    DebtTermCategory::LongTermAfrTermOver9Years => "long-term",
                },
                input.stated_principal_dollars
            ),
            citations,
            determined_issue_price_dollars: input.stated_principal_dollars,
            applicable_rate_basis_points: input.applicable_federal_rate_basis_points,
        }
    } else {
        Output {
            mode: Section1274Mode::CompliantImputedPrincipalAmountUsedAsIssuePriceSection1274A2AndB,
            statutory_basis: "IRC § 1274(a)(2) + (b) — stated interest below AFR; issue price = imputed principal amount".to_string(),
            notes: format!(
                "COMPLIANT: stated interest rate {} bps < applicable Federal rate {} bps ({} AFR category); ADEQUATE STATED INTEREST does NOT exist; § 1274(a)(2) and § 1274(b) recompute issue price as IMPUTED PRINCIPAL AMOUNT ${} (= sum of present values of all payments using AFR compounded semiannually); the difference between stated principal ${} and imputed principal ${} is reallocated as OID under § 1273(a)(1).",
                input.stated_interest_rate_basis_points,
                input.applicable_federal_rate_basis_points,
                match input.debt_term_category {
                    DebtTermCategory::ShortTermAfrTermAtOrUnder3Years => "short-term",
                    DebtTermCategory::MidTermAfrTermOver3YearsButAtOrUnder9Years => "mid-term",
                    DebtTermCategory::LongTermAfrTermOver9Years => "long-term",
                },
                input.imputed_principal_dollars,
                input.stated_principal_dollars,
                input.imputed_principal_dollars
            ),
            citations,
            determined_issue_price_dollars: input.imputed_principal_dollars,
            applicable_rate_basis_points: input.applicable_federal_rate_basis_points,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            transaction_type: TransactionType::DebtInstrumentInExchangeForProperty,
            exception_status: ExceptionStatus::NoExceptionFullyCoveredBySection1274,
            payment_timing_status: PaymentTimingStatus::AtLeastOnePaymentDueMoreThan6MonthsAfterSale,
            debt_term_category: DebtTermCategory::MidTermAfrTermOver3YearsButAtOrUnder9Years,
            stated_interest_rate_basis_points: 500,
            applicable_federal_rate_basis_points: 450,
            stated_principal_dollars: 1_000_000,
            imputed_principal_dollars: 980_000,
            fair_market_value_of_property_dollars: 1_050_000,
            is_sale_leaseback: false,
            is_potentially_abusive_situation: false,
            total_payments_aggregate_dollars: 1_200_000,
        }
    }

    #[test]
    fn not_debt_instrument_for_property_not_applicable() {
        let mut input = baseline_input();
        input.transaction_type = TransactionType::OtherTransactionNotCoveredBySection1274;
        let output = check(&input);
        assert_eq!(output.mode, Section1274Mode::NotApplicableNotDebtInstrumentForProperty);
    }

    #[test]
    fn all_payments_within_6_months_not_applicable() {
        let mut input = baseline_input();
        input.payment_timing_status =
            PaymentTimingStatus::AllPaymentsWithin6MonthsOfSaleSection1274C1RequirementNotMet;
        let output = check(&input);
        assert_eq!(output.mode, Section1274Mode::NotApplicableAllPaymentsWithin6MonthsOfSale);
    }

    #[test]
    fn farm_sale_under_1_million_exception() {
        let mut input = baseline_input();
        input.exception_status =
            ExceptionStatus::FarmSaleAtOrBelow1MillionByIndividualOrSmallBusinessSection1274C3A;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::NotApplicableExceptionFarmSaleUnder1MillionSection1274C3A
        );
    }

    #[test]
    fn principal_residence_exception() {
        let mut input = baseline_input();
        input.exception_status = ExceptionStatus::PrincipalResidenceSaleSection1274C3B;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::NotApplicableExceptionPrincipalResidenceSection1274C3B
        );
    }

    #[test]
    fn total_payments_under_250000_exception() {
        let mut input = baseline_input();
        input.exception_status = ExceptionStatus::TotalPaymentsAtOrBelow250000DollarsSection1274C3C;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::NotApplicableExceptionTotalPaymentsAtOrBelow250000Section1274C3C
        );
    }

    #[test]
    fn publicly_traded_debt_exception() {
        let mut input = baseline_input();
        input.exception_status = ExceptionStatus::PubliclyTradedDebtInstrumentSection1274C3D;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::NotApplicableExceptionPubliclyTradedDebtSection1274C3D
        );
    }

    #[test]
    fn patent_transfer_exception() {
        let mut input = baseline_input();
        input.exception_status =
            ExceptionStatus::PatentTransferUnderSection1235Section1274C3E;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::NotApplicableExceptionPatentTransferSection1274C3E
        );
    }

    #[test]
    fn potentially_abusive_situation_fmv_governs() {
        let mut input = baseline_input();
        input.is_potentially_abusive_situation = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantPotentiallyAbusiveSituationFmvOfPropertyGovernsSection1274B3
        );
        assert_eq!(output.determined_issue_price_dollars, 1_050_000);
    }

    #[test]
    fn sale_leaseback_uses_110_percent_afr() {
        let mut input = baseline_input();
        input.is_sale_leaseback = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantSaleLeasebackUses110PctOfAfrSection1274E
        );
        // 450 bps × 110% = 495 bps
        assert_eq!(output.applicable_rate_basis_points, 495);
    }

    #[test]
    fn adequate_stated_interest_uses_stated_principal() {
        // Stated 500 bps ≥ AFR 450 bps
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantAdequateStatedInterestIssuePriceEqualsStatedPrincipalSection1274A1
        );
        assert_eq!(output.determined_issue_price_dollars, 1_000_000);
    }

    #[test]
    fn stated_interest_at_exactly_afr_boundary_adequate() {
        let mut input = baseline_input();
        input.stated_interest_rate_basis_points = 450; // = AFR (boundary)
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantAdequateStatedInterestIssuePriceEqualsStatedPrincipalSection1274A1
        );
    }

    #[test]
    fn stated_interest_below_afr_uses_imputed_principal() {
        let mut input = baseline_input();
        input.stated_interest_rate_basis_points = 400; // < AFR 450
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantImputedPrincipalAmountUsedAsIssuePriceSection1274A2AndB
        );
        assert_eq!(output.determined_issue_price_dollars, 980_000);
    }

    #[test]
    fn short_term_afr_for_term_under_3_years() {
        let mut input = baseline_input();
        input.debt_term_category = DebtTermCategory::ShortTermAfrTermAtOrUnder3Years;
        input.stated_interest_rate_basis_points = 400;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1274Mode::CompliantImputedPrincipalAmountUsedAsIssuePriceSection1274A2AndB
        );
        assert!(output.notes.contains("short-term"));
    }

    #[test]
    fn mid_term_afr_for_term_between_3_and_9_years() {
        let output = check(&baseline_input());
        assert!(output.notes.contains("mid-term"));
    }

    #[test]
    fn long_term_afr_for_term_over_9_years() {
        let mut input = baseline_input();
        input.debt_term_category = DebtTermCategory::LongTermAfrTermOver9Years;
        let output = check(&input);
        assert!(output.notes.contains("long-term"));
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1274_SHORT_TERM_AFR_MAX_YEARS, 3);
        assert_eq!(IRC_1274_MID_TERM_AFR_MAX_YEARS, 9);
        assert_eq!(IRC_1274_TOTAL_PAYMENTS_EXCEPTION_THRESHOLD_DOLLARS, 250_000);
        assert_eq!(IRC_1274_FARM_SALE_EXCEPTION_THRESHOLD_DOLLARS, 1_000_000);
        assert_eq!(IRC_1274_SALE_LEASEBACK_AFR_MULTIPLIER_BASIS_POINTS, 11_000);
        assert_eq!(IRC_1274_AFR_LOOKBACK_MONTHS, 3);
        assert_eq!(IRC_1274_PAYMENT_TIMING_THRESHOLD_MONTHS, 6);
        assert_eq!(IRC_1274_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1274(a)"));
        assert!(joined.contains("§ 1274(b)"));
        assert!(joined.contains("§ 1274(c)"));
        assert!(joined.contains("§ 1274(d)"));
        assert!(joined.contains("§ 1274(e)"));
        assert!(joined.contains("SHORT-TERM AFR"));
        assert!(joined.contains("MID-TERM AFR"));
        assert!(joined.contains("LONG-TERM AFR"));
        assert!(joined.contains("3 years"));
        assert!(joined.contains("9 years"));
        assert!(joined.contains("§ 483"));
        assert!(joined.contains("§ 1273"));
        assert!(joined.contains("110 PERCENT"));
        assert!(joined.contains("$1,000,000"));
        assert!(joined.contains("$250,000"));
    }

    #[test]
    fn saturating_overflow_defense() {
        let mut input = baseline_input();
        input.applicable_federal_rate_basis_points = u64::MAX;
        input.is_sale_leaseback = true;
        let output = check(&input);
        let _ = output.mode;
    }
}
