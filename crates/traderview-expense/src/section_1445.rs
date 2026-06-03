//! IRC § 1445 — Withholding of Tax on Dispositions of United States
//! Real Property Interests (FIRPTA Withholding).
//!
//! Pure-compute FIRPTA withholding obligation under the Foreign
//! Investment in Real Property Tax Act of 1980. Buyer (transferee)
//! of a USRPI from a foreign person is statutorily required to
//! deduct and withhold a percentage of the amount realized and
//! remit to IRS on Form 8288/8288-A within 20 days of closing.
//!
//! Statute (verbatim mapping):
//! - § 1445(a) — GENERAL RULE: in any disposition of a U.S. real
//!   property interest by a foreign person, the transferee shall
//!   deduct and withhold a tax equal to 15 percent of the amount
//!   realized on the disposition.
//! - § 1445(b)(2) — non-foreign affidavit: no withholding if
//!   transferor furnishes the transferee an affidavit stating,
//!   under penalty of perjury, the transferor's TIN and that the
//!   transferor is not a foreign person.
//! - § 1445(b)(4) — publicly traded stock: no withholding if the
//!   property disposed of is an interest in a domestic corporation
//!   any class of stock of which is regularly traded on an
//!   established securities market.
//! - § 1445(b)(5) — domestically controlled REIT / RIC interest
//!   disposition not subject to § 1445(a) withholding.
//! - § 1445(b)(6) — REDUCED / ZERO RATE for buyer-residence: no
//!   withholding when the buyer acquires the property for use as
//!   a residence AND the amount realized is not more than $300,000;
//!   buyer (transferee) or member of buyer's family must have
//!   definite plans to reside at the property for at least 50 % of
//!   the number of days the property is used by any person during
//!   each of the first two 12-month periods following the date of
//!   transfer.
//! - § 1445(c)(4) — IRS withholding certificate procedure
//!   (Form 8288-B): IRS may issue a certificate reducing or
//!   eliminating withholding if the maximum tax liability is less
//!   than 15 % of amount realized, or no tax is due.
//! - PATH Act of 2015 (effective Feb 17, 2016): raised statutory
//!   rate from 10 % to 15 % under § 1445(a) (formerly 10 %); created
//!   tiered structure for buyer-residence transactions:
//!   - amount realized ≤ $300,000 + buyer-residence affidavit = 0 %
//!   - $300,000 < amount realized ≤ $1,000,000 + buyer-residence
//!     affidavit = 10 %
//!   - amount realized > $1,000,000 OR no buyer-residence affidavit
//!     = 15 %.
//!
//! Web research (verified 2026-06-03):
//! - IRS FIRPTA Withholding page: confirms 15 % default rate; 0 %
//!   under $300K + residence; 10 % $300K-$1M + residence.
//! - Cornell LII § 1445: full statutory text.
//! - IRS Exceptions page: enumerates non-foreign affidavit, publicly
//!   traded stock, withholding certificate, treaty.
//! - American Expat CPA FIRPTA Withholding Guide 2026: confirms
//!   2026 tiered structure unchanged from PATH Act.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const FIRPTA_FULL_WITHHOLDING_BASIS_POINTS: u64 = 1_500;
pub const FIRPTA_REDUCED_RESIDENCE_WITHHOLDING_BASIS_POINTS: u64 = 1_000;
pub const FIRPTA_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const FIRPTA_RESIDENCE_ZERO_RATE_THRESHOLD_DOLLARS: u64 = 300_000;
pub const FIRPTA_RESIDENCE_REDUCED_RATE_CEILING_DOLLARS: u64 = 1_000_000;
pub const FIRPTA_BUYER_RESIDENCE_MIN_USE_PCT_BASIS_POINTS: u64 = 5_000;
pub const FIRPTA_BUYER_RESIDENCE_FIRST_TWO_TWELVE_MONTH_PERIODS: u32 = 2;
pub const FIRPTA_PATH_ACT_EFFECTIVE_YEAR: u32 = 2016;
pub const FIRPTA_REPORT_FORM_8288_DEADLINE_DAYS: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferorStatus {
    UsPerson,
    ForeignPerson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    DirectUsRealPropertyInterest,
    InterestInDomesticCorporationRegularlyTradedStock,
    InterestInDomesticallyControlledReit,
    InterestInPrivatelyHeldUsRealPropertyHoldingCorporation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuyerResidenceIntent {
    NoBuyerResidenceAffidavit,
    BuyerResidenceAffidavitWithGenuinePlans,
    BuyerResidenceAffidavitFraudulent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WithholdingCertificate {
    NoneFiled,
    IssuedReducedRate,
    IssuedZeroRate,
    PendingNotYetIssued,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1445Mode {
    NotApplicableUsTransferor,
    NotApplicablePubliclyTradedStock,
    NotApplicableDomesticallyControlledReit,
    CompliantZeroRateBuyerResidenceUnder300k,
    CompliantReducedRateBuyerResidence300kTo1m,
    CompliantFullRateWithholdingApplied,
    CompliantIrsWithholdingCertificateReducedRate,
    CompliantIrsWithholdingCertificateZeroRate,
    ViolationBuyerFailedToWithholdFromForeignTransferor,
    ViolationBuyerResidenceAffidavitFraudulent,
    ViolationBuyerWithheldLessThanStatutoryRate,
    ViolationForm8288NotFiledWithin20Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub transferor_status: TransferorStatus,
    pub property_type: PropertyType,
    pub amount_realized_dollars: u64,
    pub non_foreign_affidavit_furnished: bool,
    pub buyer_residence_intent: BuyerResidenceIntent,
    pub withholding_certificate: WithholdingCertificate,
    pub actual_withheld_dollars: u64,
    pub form_8288_filed_within_20_days: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1445Mode,
    pub required_withholding_dollars: u64,
    pub applicable_rate_basis_points: u64,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section1445Input = Input;
pub type Section1445Output = Output;
pub type Section1445Result = Output;

fn apply_rate(amount_dollars: u64, rate_bp: u64) -> u64 {
    (amount_dollars as u128)
        .saturating_mul(rate_bp as u128)
        .checked_div(FIRPTA_BASIS_POINT_DENOMINATOR as u128)
        .unwrap_or(0) as u64
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 1445(a) — general rule: 15 % withholding on amount realized on USRPI disposition by foreign person".to_string(),
        "26 U.S.C. § 1445(b)(2) — non-foreign affidavit exception (transferor TIN + under penalty of perjury statement)".to_string(),
        "26 U.S.C. § 1445(b)(4) — publicly traded stock exception".to_string(),
        "26 U.S.C. § 1445(b)(5) — domestically controlled REIT exception".to_string(),
        "26 U.S.C. § 1445(b)(6) — buyer-residence exception: ≤ $300,000 + 50 % residence affidavit = 0 % withholding".to_string(),
        "26 U.S.C. § 1445(c)(4) — IRS withholding certificate reducing/eliminating withholding (Form 8288-B)".to_string(),
        "PATH Act of 2015 (P.L. 114-113) § 324 — raised statutory rate from 10 % to 15 % effective Feb 17, 2016; created tiered residence rate structure ($300K / $1M)".to_string(),
        "Treas. Reg. § 1.1445-2 — situations in which withholding is not required under § 1445(a)".to_string(),
        "Form 8288 — U.S. Withholding Tax Return for Dispositions by Foreign Persons of U.S. Real Property Interests; due within 20 days of closing".to_string(),
        "Form 8288-A — Statement of Withholding furnished to transferor (foreign seller)".to_string(),
        "Form 8288-B — Application for Withholding Certificate to reduce or eliminate withholding".to_string(),
    ];

    if input.transferor_status == TransferorStatus::UsPerson || input.non_foreign_affidavit_furnished {
        return Output {
            mode: Section1445Mode::NotApplicableUsTransferor,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1445(b)(2) — US transferor or non-foreign affidavit furnished".to_string(),
            notes: format!(
                "No § 1445 withholding required: transferor is {:?} or furnished non-foreign affidavit. Amount realized = ${}.",
                input.transferor_status, input.amount_realized_dollars
            ),
            citations,
        };
    }

    if input.property_type == PropertyType::InterestInDomesticCorporationRegularlyTradedStock {
        return Output {
            mode: Section1445Mode::NotApplicablePubliclyTradedStock,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1445(b)(4) — publicly traded stock exception".to_string(),
            notes: "No § 1445 withholding: property is interest in domestic corporation with regularly traded stock.".to_string(),
            citations,
        };
    }

    if input.property_type == PropertyType::InterestInDomesticallyControlledReit {
        return Output {
            mode: Section1445Mode::NotApplicableDomesticallyControlledReit,
            required_withholding_dollars: 0,
            applicable_rate_basis_points: 0,
            statutory_basis: "§ 1445(b)(5) — domestically controlled REIT exception".to_string(),
            notes: "No § 1445 withholding: domestically controlled REIT/RIC interest disposition.".to_string(),
            citations,
        };
    }

    if input.buyer_residence_intent == BuyerResidenceIntent::BuyerResidenceAffidavitFraudulent {
        let required = apply_rate(input.amount_realized_dollars, FIRPTA_FULL_WITHHOLDING_BASIS_POINTS);
        return Output {
            mode: Section1445Mode::ViolationBuyerResidenceAffidavitFraudulent,
            required_withholding_dollars: required,
            applicable_rate_basis_points: FIRPTA_FULL_WITHHOLDING_BASIS_POINTS,
            statutory_basis: "§ 1445(a) — fraudulent residence affidavit voids § 1445(b)(6) exception".to_string(),
            notes: format!(
                "VIOLATION: buyer residence affidavit is fraudulent (no genuine 50 % use plan). § 1445(b)(6) exception unavailable. Default 15 % rate applies to amount realized ${} = ${} required withholding.",
                input.amount_realized_dollars, required
            ),
            citations,
        };
    }

    match input.withholding_certificate {
        WithholdingCertificate::IssuedZeroRate => {
            return Output {
                mode: Section1445Mode::CompliantIrsWithholdingCertificateZeroRate,
                required_withholding_dollars: 0,
                applicable_rate_basis_points: 0,
                statutory_basis: "§ 1445(c)(4) — IRS withholding certificate issued at 0 % rate".to_string(),
                notes: format!(
                    "COMPLIANT: IRS withholding certificate (Form 8288-B) issued at 0 % rate. No withholding required on amount realized ${}.",
                    input.amount_realized_dollars
                ),
                citations,
            };
        }
        WithholdingCertificate::IssuedReducedRate => {
            return Output {
                mode: Section1445Mode::CompliantIrsWithholdingCertificateReducedRate,
                required_withholding_dollars: input.actual_withheld_dollars,
                applicable_rate_basis_points: 0,
                statutory_basis: "§ 1445(c)(4) — IRS withholding certificate issued at reduced rate".to_string(),
                notes: format!(
                    "COMPLIANT: IRS withholding certificate (Form 8288-B) issued at reduced rate. Actual withholding ${} accepted per certificate on amount realized ${}.",
                    input.actual_withheld_dollars, input.amount_realized_dollars
                ),
                citations,
            };
        }
        _ => {}
    }

    let buyer_residence_affidavit_valid = input.buyer_residence_intent
        == BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans;

    let (rate_bp, mode_if_compliant) = if buyer_residence_affidavit_valid
        && input.amount_realized_dollars <= FIRPTA_RESIDENCE_ZERO_RATE_THRESHOLD_DOLLARS
    {
        (0, Section1445Mode::CompliantZeroRateBuyerResidenceUnder300k)
    } else if buyer_residence_affidavit_valid
        && input.amount_realized_dollars <= FIRPTA_RESIDENCE_REDUCED_RATE_CEILING_DOLLARS
    {
        (
            FIRPTA_REDUCED_RESIDENCE_WITHHOLDING_BASIS_POINTS,
            Section1445Mode::CompliantReducedRateBuyerResidence300kTo1m,
        )
    } else {
        (
            FIRPTA_FULL_WITHHOLDING_BASIS_POINTS,
            Section1445Mode::CompliantFullRateWithholdingApplied,
        )
    };

    let required = apply_rate(input.amount_realized_dollars, rate_bp);

    if input.actual_withheld_dollars == 0 && required > 0 {
        return Output {
            mode: Section1445Mode::ViolationBuyerFailedToWithholdFromForeignTransferor,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1445(a) — buyer (transferee) is statutory withholding agent and personally liable for tax not withheld".to_string(),
            notes: format!(
                "VIOLATION § 1445(a): buyer failed to withhold from foreign transferor on amount realized ${}. Required withholding at {} basis points = ${}. Buyer personally liable for unwithheld tax + § 6651 / § 6656 penalties + interest.",
                input.amount_realized_dollars, rate_bp, required
            ),
            citations,
        };
    }

    if input.actual_withheld_dollars < required {
        return Output {
            mode: Section1445Mode::ViolationBuyerWithheldLessThanStatutoryRate,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1445(a) — actual withholding less than required statutory rate".to_string(),
            notes: format!(
                "VIOLATION § 1445(a): buyer withheld ${} (less than required ${}) on amount realized ${} at applicable rate {} basis points. Shortfall ${} subject to buyer personal liability.",
                input.actual_withheld_dollars,
                required,
                input.amount_realized_dollars,
                rate_bp,
                required.saturating_sub(input.actual_withheld_dollars)
            ),
            citations,
        };
    }

    if required > 0 && !input.form_8288_filed_within_20_days {
        return Output {
            mode: Section1445Mode::ViolationForm8288NotFiledWithin20Days,
            required_withholding_dollars: required,
            applicable_rate_basis_points: rate_bp,
            statutory_basis: "§ 1445(a) + Form 8288 filing rule: 20-day deadline missed".to_string(),
            notes: format!(
                "VIOLATION: withholding amount ${} computed correctly but Form 8288 not filed within 20 days of closing. Late filing subject to § 6651(a)(1) failure-to-file penalty + interest.",
                required
            ),
            citations,
        };
    }

    Output {
        mode: mode_if_compliant,
        required_withholding_dollars: required,
        applicable_rate_basis_points: rate_bp,
        statutory_basis: match mode_if_compliant {
            Section1445Mode::CompliantZeroRateBuyerResidenceUnder300k => {
                "§ 1445(b)(6) — buyer-residence + amount realized ≤ $300,000 = 0 % rate".to_string()
            }
            Section1445Mode::CompliantReducedRateBuyerResidence300kTo1m => {
                "§ 1445(b)(6) + PATH Act § 324 — buyer-residence + $300,001-$1,000,000 = 10 % rate".to_string()
            }
            _ => "§ 1445(a) — default 15 % statutory rate".to_string(),
        },
        notes: format!(
            "COMPLIANT § 1445: amount realized ${}, applicable rate {} basis points, required withholding ${}, actual withholding ${}, Form 8288 filed timely.",
            input.amount_realized_dollars, rate_bp, required, input.actual_withheld_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_foreign_seller_full_rate() -> Input {
        Input {
            transferor_status: TransferorStatus::ForeignPerson,
            property_type: PropertyType::DirectUsRealPropertyInterest,
            amount_realized_dollars: 2_000_000,
            non_foreign_affidavit_furnished: false,
            buyer_residence_intent: BuyerResidenceIntent::NoBuyerResidenceAffidavit,
            withholding_certificate: WithholdingCertificate::NoneFiled,
            actual_withheld_dollars: 300_000,
            form_8288_filed_within_20_days: true,
        }
    }

    #[test]
    fn us_person_transferor_not_applicable() {
        let input = Input {
            transferor_status: TransferorStatus::UsPerson,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::NotApplicableUsTransferor);
        assert_eq!(result.required_withholding_dollars, 0);
    }

    #[test]
    fn non_foreign_affidavit_furnished_not_applicable() {
        let input = Input {
            non_foreign_affidavit_furnished: true,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::NotApplicableUsTransferor);
    }

    #[test]
    fn publicly_traded_stock_not_applicable() {
        let input = Input {
            property_type: PropertyType::InterestInDomesticCorporationRegularlyTradedStock,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::NotApplicablePubliclyTradedStock);
    }

    #[test]
    fn domestically_controlled_reit_not_applicable() {
        let input = Input {
            property_type: PropertyType::InterestInDomesticallyControlledReit,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::NotApplicableDomesticallyControlledReit);
    }

    #[test]
    fn buyer_residence_under_300k_zero_rate_compliant() {
        let input = Input {
            amount_realized_dollars: 280_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans,
            actual_withheld_dollars: 0,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantZeroRateBuyerResidenceUnder300k);
        assert_eq!(result.required_withholding_dollars, 0);
        assert_eq!(result.applicable_rate_basis_points, 0);
    }

    #[test]
    fn buyer_residence_at_exactly_300k_zero_rate_compliant() {
        let input = Input {
            amount_realized_dollars: 300_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans,
            actual_withheld_dollars: 0,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantZeroRateBuyerResidenceUnder300k);
    }

    #[test]
    fn buyer_residence_500k_reduced_10_pct_compliant() {
        let input = Input {
            amount_realized_dollars: 500_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans,
            actual_withheld_dollars: 50_000,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantReducedRateBuyerResidence300kTo1m);
        assert_eq!(result.required_withholding_dollars, 50_000);
        assert_eq!(result.applicable_rate_basis_points, 1_000);
    }

    #[test]
    fn buyer_residence_at_exactly_1m_reduced_10_pct_compliant() {
        let input = Input {
            amount_realized_dollars: 1_000_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans,
            actual_withheld_dollars: 100_000,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantReducedRateBuyerResidence300kTo1m);
    }

    #[test]
    fn buyer_residence_above_1m_full_15_pct_compliant() {
        let input = Input {
            amount_realized_dollars: 1_500_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitWithGenuinePlans,
            actual_withheld_dollars: 225_000,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantFullRateWithholdingApplied);
        assert_eq!(result.required_withholding_dollars, 225_000);
        assert_eq!(result.applicable_rate_basis_points, 1_500);
    }

    #[test]
    fn default_15_pct_full_rate_compliant() {
        let result = compute(&baseline_foreign_seller_full_rate());
        assert_eq!(result.mode, Section1445Mode::CompliantFullRateWithholdingApplied);
        assert_eq!(result.required_withholding_dollars, 300_000);
    }

    #[test]
    fn buyer_failed_to_withhold_violation() {
        let input = Input {
            actual_withheld_dollars: 0,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::ViolationBuyerFailedToWithholdFromForeignTransferor);
        assert_eq!(result.required_withholding_dollars, 300_000);
    }

    #[test]
    fn buyer_withheld_less_than_required_violation() {
        let input = Input {
            actual_withheld_dollars: 100_000,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::ViolationBuyerWithheldLessThanStatutoryRate);
        assert_eq!(result.required_withholding_dollars, 300_000);
        assert!(result.notes.contains("Shortfall $200000"));
    }

    #[test]
    fn fraudulent_residence_affidavit_violation() {
        let input = Input {
            amount_realized_dollars: 250_000,
            buyer_residence_intent: BuyerResidenceIntent::BuyerResidenceAffidavitFraudulent,
            actual_withheld_dollars: 37_500,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::ViolationBuyerResidenceAffidavitFraudulent);
        assert_eq!(result.required_withholding_dollars, 37_500);
        assert_eq!(result.applicable_rate_basis_points, 1_500);
    }

    #[test]
    fn form_8288_late_filing_violation() {
        let input = Input {
            form_8288_filed_within_20_days: false,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::ViolationForm8288NotFiledWithin20Days);
    }

    #[test]
    fn irs_withholding_certificate_zero_rate_compliant() {
        let input = Input {
            withholding_certificate: WithholdingCertificate::IssuedZeroRate,
            actual_withheld_dollars: 0,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantIrsWithholdingCertificateZeroRate);
        assert_eq!(result.required_withholding_dollars, 0);
    }

    #[test]
    fn irs_withholding_certificate_reduced_rate_compliant() {
        let input = Input {
            withholding_certificate: WithholdingCertificate::IssuedReducedRate,
            actual_withheld_dollars: 50_000,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert_eq!(result.mode, Section1445Mode::CompliantIrsWithholdingCertificateReducedRate);
        assert_eq!(result.required_withholding_dollars, 50_000);
    }

    #[test]
    fn citations_pin_section_1445_subsections_and_forms() {
        let result = compute(&baseline_foreign_seller_full_rate());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 1445(a)"));
        assert!(joined.contains("§ 1445(b)(2)"));
        assert!(joined.contains("§ 1445(b)(4)"));
        assert!(joined.contains("§ 1445(b)(5)"));
        assert!(joined.contains("§ 1445(b)(6)"));
        assert!(joined.contains("§ 1445(c)(4)"));
        assert!(joined.contains("PATH Act"));
        assert!(joined.contains("Form 8288"));
        assert!(joined.contains("Form 8288-A"));
        assert!(joined.contains("Form 8288-B"));
    }

    #[test]
    fn constant_pin_rates_and_thresholds() {
        assert_eq!(FIRPTA_FULL_WITHHOLDING_BASIS_POINTS, 1_500);
        assert_eq!(FIRPTA_REDUCED_RESIDENCE_WITHHOLDING_BASIS_POINTS, 1_000);
        assert_eq!(FIRPTA_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(FIRPTA_RESIDENCE_ZERO_RATE_THRESHOLD_DOLLARS, 300_000);
        assert_eq!(FIRPTA_RESIDENCE_REDUCED_RATE_CEILING_DOLLARS, 1_000_000);
        assert_eq!(FIRPTA_BUYER_RESIDENCE_MIN_USE_PCT_BASIS_POINTS, 5_000);
        assert_eq!(FIRPTA_BUYER_RESIDENCE_FIRST_TWO_TWELVE_MONTH_PERIODS, 2);
        assert_eq!(FIRPTA_REPORT_FORM_8288_DEADLINE_DAYS, 20);
    }

    #[test]
    fn constant_pin_path_act_2016_effective_year() {
        assert_eq!(FIRPTA_PATH_ACT_EFFECTIVE_YEAR, 2016);
    }

    #[test]
    fn saturating_overflow_defense_extreme_amount_realized() {
        let input = Input {
            amount_realized_dollars: u64::MAX,
            actual_withheld_dollars: u64::MAX,
            ..baseline_foreign_seller_full_rate()
        };
        let result = compute(&input);
        assert!(matches!(
            result.mode,
            Section1445Mode::CompliantFullRateWithholdingApplied
                | Section1445Mode::ViolationBuyerWithheldLessThanStatutoryRate
        ));
    }
}
