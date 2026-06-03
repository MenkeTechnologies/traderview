//! IRC § 1286 — Tax Treatment of Stripped Bonds /
//! Stripped Coupons / Bond Stripping OID Inclusion Module.
//!
//! Pure-compute check for IRC § 1286 OID-inclusion treatment
//! of stripped bonds and stripped coupons. § 1286 is the
//! TEFRA-era anti-abuse rule that ended pre-1982 trader
//! ability to manufacture artificial losses by purchasing a
//! coupon bond, stripping the coupons, and selling the bond
//! (without coupons) for less than basis. § 1286 was enacted
//! as the successor to predecessor § 1232B by the Tax Equity
//! and Fiscal Responsibility Act of 1982 (Public Law 97-248)
//! and applies to stripped bonds purchased AFTER JULY 1, 1982.
//! The Tax Reform Act of 1986 amended § 1286 to bring
//! stripped tax-exempt obligations within scope effective for
//! purchases or sales AFTER JUNE 10, 1987 — closing the
//! parallel municipal-stripping loophole.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 1286(a) Inclusion in Income of Original Issue
//!   Discount**: a stripped bond or stripped coupon purchased
//!   AFTER JULY 1, 1982 is treated by the purchaser as a bond
//!   ORIGINALLY ISSUED on the purchase date and having ORIGINAL
//!   ISSUE DISCOUNT equal to the EXCESS of the **STATED
//!   REDEMPTION PRICE AT MATURITY (SRPM)** OVER the **RATABLE
//!   SHARE of the PURCHASE PRICE** of the stripped bond or
//!   coupon ([Cornell LII 26 USC § 1286](https://www.law.cornell.edu/uscode/text/26/1286);
//!   [Bloomberg Tax Sec. 1286](https://irc.bloombergtax.com/public/uscode/doc/irc/section_1286)).
//! - **IRC § 1286(b) Tax Treatment of Person Stripping
//!   Coupons**: if any person strips one or more coupons from
//!   a bond and disposes of the bond OR a coupon, then
//!   immediately before the disposition the stripping person
//!   MUST include in gross income (1) interest accrued on the
//!   bond while held by such person and not previously included
//!   in income, AND (2) accrued market discount on the bond.
//!   The basis of the bond is INCREASED by the amount of
//!   interest and market discount so included, then the basis
//!   is ALLOCATED among the bond and coupons in proportion to
//!   their respective FAIR MARKET VALUES at the time of
//!   disposition.
//! - **IRC § 1286(c)/(d) Tax-Exempt Obligation Stripping Rules**
//!   added by Tax Reform Act of 1986 § 1879 effective for
//!   purchases or sales AFTER JUNE 10, 1987: in the case of a
//!   stripped tax-exempt obligation, the OID determined under
//!   § 1286(a) is split into TWO COMPONENTS — (i) the
//!   **TAX-EXEMPT PORTION** limited to the OID accruing at a
//!   YIELD equal to the LOWER of (A) the coupon rate of
//!   interest on the obligation OR (B) the stripped
//!   obligation's yield to maturity, AND (ii) the
//!   **NON-EXEMPT PORTION** subject to ordinary OID inclusion
//!   ([26 CFR § 1.1286-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR56edfa33b27e3cf/section-1.1286-1)).
//! - **§ 1273(a)(3) De Minimis OID Rule (incorporated by
//!   § 1286)**: if the OID determined under § 1286(a) with
//!   respect to the purchase of a stripped bond or stripped
//!   coupon is LESS THAN the amount computed under the OID
//!   de minimis rule of § 1273(a)(3) — namely **0.25 % ×
//!   STATED REDEMPTION PRICE AT MATURITY × NUMBER OF COMPLETE
//!   YEARS TO MATURITY from the issue date** — the amount of
//!   OID is treated as ZERO and no current OID inclusion is
//!   required.
//! - **IRC § 1286(d) Definitions**: (1) "bond" means a bond,
//!   debenture, note, certificate, or other evidence of
//!   indebtedness; (2) "stripped bond" means a bond issued at
//!   any time with INTEREST COUPONS where there has been a
//!   SEPARATION IN OWNERSHIP between the bond and any coupon;
//!   (3) "stripped coupon" means any coupon relating to a
//!   stripped bond (including the right to receive interest);
//!   (4) "stated redemption price at maturity" has the same
//!   meaning as in § 1273(a)(2); (5) "coupon" includes any
//!   right to receive interest on a bond whether or not
//!   evidenced by an old-style detachable coupon and whether
//!   or not the bond is in coupon-bearer form; (6) "purchase"
//!   has the same meaning as in § 1272(d)(1).
//! - **Predecessor § 1232B**: § 1286 succeeded predecessor
//!   IRC § 1232B which was adopted as part of TEFRA
//!   (Public Law 97-248) effective for stripped bonds and
//!   coupons purchased AFTER JULY 1, 1982 to close the
//!   pre-TEFRA strip-and-sell artificial-loss loophole; the
//!   1984 codification reorganized § 1232B as § 1286
//!   without substantive change.
//! - **Companion Provisions**: § 1271 (treatment of amounts
//!   received on retirement); § 1272 (current inclusion of OID
//!   in income); § 1273 (determination of amount of OID +
//!   de minimis rule); § 1274 (issue price determination for
//!   debt-for-property exchanges); § 1275 (OID definitions +
//!   special rules); § 1288 (anti-avoidance tax-exempt OID
//!   special rules).
//! - **Trader / Fixed-Income-Desk Significance**: every
//!   trader who buys zero-coupon Treasury STRIPS, TIPS strips,
//!   coupon-stripped municipal bonds, or any stripped corporate
//!   debt must accrue OID currently under § 1286(a) — the
//!   stripped instrument is treated as having NO coupons and
//!   FULL OID equal to (SRPM − purchase price). The 0.25 %
//!   per year de minimis rule under § 1273(a)(3) provides a
//!   narrow escape for trivial OID amounts but does NOT apply
//!   to standard STRIPS where OID is the entire return.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_1286_EFFECTIVE_DATE_YEAR: u32 = 1982;
pub const IRC_1286_EFFECTIVE_DATE_MONTH: u32 = 7;
pub const IRC_1286_EFFECTIVE_DATE_DAY: u32 = 1;
pub const IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_YEAR: u32 = 1987;
pub const IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_MONTH: u32 = 6;
pub const IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_DAY: u32 = 10;
pub const IRC_1273_A3_DE_MINIMIS_BASIS_POINTS_PER_YEAR: u64 = 25;
pub const IRC_1286_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerRole {
    StripperOfCouponsFromBond,
    PurchaserOfStrippedBondOrCoupon,
    NotInvolvedInStrippingTransaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationType {
    TaxableBond,
    TaxExemptObligation,
    ShortTermObligationLessThanOneYearAtIssue,
    NotAStrippedBondOrCoupon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseDateStatus {
    PurchasedAfterJuly1_1982PostTefraEffective,
    PurchasedOnOrBeforeJuly1_1982PreTefraEffective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxExemptAmendmentDateStatus {
    PurchasedAfterJune10_1987PostTraAmendmentEffective,
    PurchasedOnOrBeforeJune10_1987PreTraAmendmentEffective,
    NotApplicableNotTaxExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StripperAction {
    StripperIncludedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv,
    StripperDidNotIncludeAccruedInterestAndMarketDiscount,
    StripperDidNotAllocateBasisByFmv,
    NotApplicableNotStripper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaserAction {
    PurchaserRecognizedOidInclusionPostJuly1_1982,
    PurchaserDidNotRecognizeOid,
    TaxExemptOidProperlySplitTaxExemptAndTaxablePortions,
    TaxExemptOidNotProperlySplitTaxExemptVsTaxablePortion,
    NotApplicableNotPurchaser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1286Mode {
    NotApplicableObligationPurchasedOnOrBeforeJuly1_1982PreTefraEffective,
    NotApplicableTaxExemptStrippedObligationPurchasedOnOrBeforeJune10_1987PreAmendment,
    NotApplicableObligationIsNotAStrippedBondOrCoupon,
    NotApplicableShortTermObligationLessThanOneYearAtIssue,
    NotApplicableNotInvolvedInStrippingTransaction,
    CompliantStripperRecognizedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv,
    CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982,
    CompliantTaxExemptStrippedObligationOidProperlySplitPerSection1286C,
    CompliantOidUnderDeMinimisThresholdSection1273A3NoCurrentInclusionRequired,
    ViolationStripperFailedToIncludeAccruedInterestAndMarketDiscount,
    ViolationStripperFailedToAllocateBasisByFairMarketValue,
    ViolationPurchaserFailedToRecognizeOidOnStrippedBondPostJuly1_1982,
    ViolationTaxExemptStrippedObligationOidNotProperlySplitPostJune10_1987,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub taxpayer_role: TaxpayerRole,
    pub obligation_type: ObligationType,
    pub purchase_date_status: PurchaseDateStatus,
    pub tax_exempt_amendment_date_status: TaxExemptAmendmentDateStatus,
    pub stripper_action: StripperAction,
    pub purchaser_action: PurchaserAction,
    pub stated_redemption_price_at_maturity_dollars: u64,
    pub ratable_share_of_purchase_price_dollars: u64,
    pub years_to_maturity: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section1286Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub computed_oid_dollars: u64,
    pub de_minimis_threshold_dollars: u64,
}

pub type Section1286Input = Input;
pub type Section1286Output = Output;
pub type Section1286Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 1286(a) Inclusion in Income of Original Issue Discount — stripped bond or stripped coupon purchased AFTER JULY 1, 1982 is treated by purchaser as a bond ORIGINALLY ISSUED on purchase date and having ORIGINAL ISSUE DISCOUNT equal to EXCESS of STATED REDEMPTION PRICE AT MATURITY (SRPM) over RATABLE SHARE of PURCHASE PRICE".to_string(),
        "IRC § 1286(b) Tax Treatment of Person Stripping Coupons — if person strips one or more coupons from bond and disposes of bond OR coupon, immediately before disposition stripping person MUST include in gross income (1) interest accrued on bond while held and not previously included in income AND (2) accrued market discount; basis of bond INCREASED by amount included, then ALLOCATED among bond and coupons in proportion to respective FAIR MARKET VALUES at time of disposition".to_string(),
        "IRC § 1286(c)/(d) Tax-Exempt Obligation Stripping Rules — added by Tax Reform Act of 1986 § 1879 effective for purchases or sales AFTER JUNE 10, 1987; stripped tax-exempt obligation OID under § 1286(a) split into TAX-EXEMPT PORTION limited to OID accruing at YIELD equal to LOWER of (A) coupon rate or (B) stripped obligation's yield to maturity, AND NON-EXEMPT PORTION subject to ordinary OID inclusion".to_string(),
        "§ 1273(a)(3) De Minimis OID Rule (incorporated by § 1286) — if OID is LESS THAN 0.25 % × STATED REDEMPTION PRICE AT MATURITY × NUMBER OF COMPLETE YEARS TO MATURITY, OID is treated as ZERO and no current inclusion required".to_string(),
        "IRC § 1286(d) Definitions — (1) 'bond' = bond, debenture, note, certificate, or other evidence of indebtedness; (2) 'stripped bond' = bond issued with interest coupons where separation in ownership has occurred between bond and any coupon; (3) 'stripped coupon' = coupon relating to stripped bond (including right to receive interest); (4) 'stated redemption price at maturity' = same meaning as § 1273(a)(2); (5) 'coupon' includes any right to receive interest on bond whether or not evidenced by detachable coupon; (6) 'purchase' = same meaning as § 1272(d)(1)".to_string(),
        "Effective Date — § 1286 applies to stripped bonds and coupons PURCHASED AFTER JULY 1, 1982 (TEFRA Public Law 97-248 enactment of predecessor § 1232B; codified 1984 as § 1286 without substantive change)".to_string(),
        "Tax-Exempt Amendment Effective Date — § 1286(c)/(d) tax-exempt stripped obligation rules apply to any PURCHASE OR SALE AFTER JUNE 10, 1987 (Tax Reform Act of 1986 § 1879)".to_string(),
        "Predecessor § 1232B — Tax Equity and Fiscal Responsibility Act of 1982 (Public Law 97-248) enacted predecessor § 1232B to close pre-TEFRA strip-and-sell artificial-loss loophole; 1984 codification reorganized § 1232B as § 1286 without substantive change".to_string(),
        "Companion Provisions — § 1271 (treatment of amounts received on retirement); § 1272 (current inclusion of OID in income); § 1273 (determination of amount of OID + de minimis rule); § 1274 (issue price determination for debt-for-property exchanges); § 1275 (OID definitions + special rules); § 1288 (anti-avoidance tax-exempt OID special rules)".to_string(),
        "Cornell LII 26 USC § 1286 — primary statutory text".to_string(),
        "Bloomberg Tax Sec. 1286 — comprehensive code commentary".to_string(),
        "26 CFR § 1.1286-1 — Treasury Regulation implementing tax treatment of certain stripped bonds and stripped coupons".to_string(),
        "Treasury Decision 8731 (1997) — final regulations on treatment of stripped bonds and stripped coupons".to_string(),
        "Tax Reform Act of 1986 § 1879 (Public Law 99-514) — added tax-exempt stripped obligation rules effective for purchases or sales after June 10, 1987".to_string(),
    ];

    if input.obligation_type == ObligationType::NotAStrippedBondOrCoupon {
        return Output {
            mode: Section1286Mode::NotApplicableObligationIsNotAStrippedBondOrCoupon,
            statutory_basis: "IRC § 1286(d)(2)/(3) — § 1286 applies only to stripped bonds and stripped coupons".to_string(),
            notes: "NOT APPLICABLE: obligation is not a stripped bond or stripped coupon within § 1286(d) definitions; § 1286 OID-inclusion treatment does not apply.".to_string(),
            citations,
            computed_oid_dollars: 0,
            de_minimis_threshold_dollars: 0,
        };
    }

    if input.obligation_type == ObligationType::ShortTermObligationLessThanOneYearAtIssue {
        return Output {
            mode: Section1286Mode::NotApplicableShortTermObligationLessThanOneYearAtIssue,
            statutory_basis: "IRC § 1286 — short-term obligation (< 1 year at issue) outside § 1286 OID inclusion scope".to_string(),
            notes: "NOT APPLICABLE: short-term obligation with less than one year to maturity at issue; § 1286 OID-inclusion treatment does not apply (short-term obligations covered by separate OID accrual rules).".to_string(),
            citations,
            computed_oid_dollars: 0,
            de_minimis_threshold_dollars: 0,
        };
    }

    if input.purchase_date_status
        == PurchaseDateStatus::PurchasedOnOrBeforeJuly1_1982PreTefraEffective
    {
        return Output {
            mode: Section1286Mode::NotApplicableObligationPurchasedOnOrBeforeJuly1_1982PreTefraEffective,
            statutory_basis: "TEFRA Public Law 97-248 effective date — § 1286 (predecessor § 1232B) applies only to stripped bonds purchased AFTER July 1, 1982".to_string(),
            notes: "NOT APPLICABLE: obligation purchased on or before July 1, 1982; pre-TEFRA grandfathered transaction; § 1286 OID-inclusion treatment does not apply.".to_string(),
            citations,
            computed_oid_dollars: 0,
            de_minimis_threshold_dollars: 0,
        };
    }

    if input.obligation_type == ObligationType::TaxExemptObligation
        && input.tax_exempt_amendment_date_status
            == TaxExemptAmendmentDateStatus::PurchasedOnOrBeforeJune10_1987PreTraAmendmentEffective
    {
        return Output {
            mode: Section1286Mode::NotApplicableTaxExemptStrippedObligationPurchasedOnOrBeforeJune10_1987PreAmendment,
            statutory_basis: "Tax Reform Act of 1986 § 1879 effective date — § 1286(c)/(d) tax-exempt stripped obligation rules apply only to purchases or sales after June 10, 1987".to_string(),
            notes: "NOT APPLICABLE: tax-exempt stripped obligation purchased on or before June 10, 1987; pre-TRA-1986-amendment grandfathered transaction; § 1286(c)/(d) tax-exempt stripped obligation rules do not apply.".to_string(),
            citations,
            computed_oid_dollars: 0,
            de_minimis_threshold_dollars: 0,
        };
    }

    if input.taxpayer_role == TaxpayerRole::NotInvolvedInStrippingTransaction {
        return Output {
            mode: Section1286Mode::NotApplicableNotInvolvedInStrippingTransaction,
            statutory_basis: "IRC § 1286 — applies only to stripper or purchaser of stripped bond/coupon".to_string(),
            notes: "NOT APPLICABLE: taxpayer is neither the person stripping coupons nor the purchaser of stripped bond/coupon; § 1286 does not apply.".to_string(),
            citations,
            computed_oid_dollars: 0,
            de_minimis_threshold_dollars: 0,
        };
    }

    let computed_oid_dollars = input
        .stated_redemption_price_at_maturity_dollars
        .saturating_sub(input.ratable_share_of_purchase_price_dollars);

    let de_minimis_threshold_dollars = u128::from(input.stated_redemption_price_at_maturity_dollars)
        .saturating_mul(u128::from(IRC_1273_A3_DE_MINIMIS_BASIS_POINTS_PER_YEAR))
        .saturating_mul(u128::from(input.years_to_maturity))
        .checked_div(u128::from(IRC_1286_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;

    if computed_oid_dollars < de_minimis_threshold_dollars && computed_oid_dollars > 0 {
        return Output {
            mode: Section1286Mode::CompliantOidUnderDeMinimisThresholdSection1273A3NoCurrentInclusionRequired,
            statutory_basis: "§ 1273(a)(3) de minimis OID rule (incorporated by § 1286) — OID < 0.25 % × SRPM × years to maturity treated as zero".to_string(),
            notes: format!(
                "COMPLIANT: computed OID {} dollars is below the § 1273(a)(3) de minimis threshold of {} dollars (= 0.25 % × SRPM {} × {} years to maturity); OID treated as ZERO; no current inclusion required.",
                computed_oid_dollars,
                de_minimis_threshold_dollars,
                input.stated_redemption_price_at_maturity_dollars,
                input.years_to_maturity
            ),
            citations,
            computed_oid_dollars,
            de_minimis_threshold_dollars,
        };
    }

    match input.taxpayer_role {
        TaxpayerRole::NotInvolvedInStrippingTransaction => {
            // Already handled above — defensive duplicate
            Output {
                mode: Section1286Mode::NotApplicableNotInvolvedInStrippingTransaction,
                statutory_basis: "IRC § 1286".to_string(),
                notes: "NOT APPLICABLE.".to_string(),
                citations,
                computed_oid_dollars,
                de_minimis_threshold_dollars,
            }
        }
        TaxpayerRole::StripperOfCouponsFromBond => match input.stripper_action {
            StripperAction::StripperIncludedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv => Output {
                mode: Section1286Mode::CompliantStripperRecognizedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv,
                statutory_basis: "IRC § 1286(b) — stripping person includes accrued interest + market discount in gross income at disposition and allocates basis by FMV".to_string(),
                notes: format!(
                    "COMPLIANT: stripper included accrued interest and market discount in gross income immediately before disposition AND allocated basis among bond and coupons in proportion to their respective fair market values; § 1286(b) requirements satisfied. Computed OID at purchase {} dollars; de minimis threshold {} dollars.",
                    computed_oid_dollars, de_minimis_threshold_dollars
                ),
                citations,
                computed_oid_dollars,
                de_minimis_threshold_dollars,
            },
            StripperAction::StripperDidNotIncludeAccruedInterestAndMarketDiscount => Output {
                mode: Section1286Mode::ViolationStripperFailedToIncludeAccruedInterestAndMarketDiscount,
                statutory_basis: "IRC § 1286(b)(1) — stripper must include accrued interest and accrued market discount in gross income".to_string(),
                notes: "VIOLATION: stripper failed to include accrued interest and accrued market discount on bond in gross income immediately before disposition; § 1286(b)(1) requires inclusion regardless of whether disposition was of bond or coupon.".to_string(),
                citations,
                computed_oid_dollars,
                de_minimis_threshold_dollars,
            },
            StripperAction::StripperDidNotAllocateBasisByFmv => Output {
                mode: Section1286Mode::ViolationStripperFailedToAllocateBasisByFairMarketValue,
                statutory_basis: "IRC § 1286(b)(2) — basis allocated among bond and coupons in proportion to fair market values".to_string(),
                notes: "VIOLATION: stripper failed to allocate basis among the bond and the coupons in proportion to their respective fair market values at the time of disposition; § 1286(b)(2) allocation rule violated.".to_string(),
                citations,
                computed_oid_dollars,
                de_minimis_threshold_dollars,
            },
            StripperAction::NotApplicableNotStripper => Output {
                mode: Section1286Mode::NotApplicableNotInvolvedInStrippingTransaction,
                statutory_basis: "IRC § 1286(b) — stripper action input does not match stripper role".to_string(),
                notes: "NOT APPLICABLE: stripper role asserted but stripper_action set to NotApplicable; treating as not involved in stripping transaction.".to_string(),
                citations,
                computed_oid_dollars,
                de_minimis_threshold_dollars,
            },
        },
        TaxpayerRole::PurchaserOfStrippedBondOrCoupon => {
            if input.obligation_type == ObligationType::TaxExemptObligation
                && input.tax_exempt_amendment_date_status
                    == TaxExemptAmendmentDateStatus::PurchasedAfterJune10_1987PostTraAmendmentEffective
            {
                return match input.purchaser_action {
                    PurchaserAction::TaxExemptOidProperlySplitTaxExemptAndTaxablePortions => Output {
                        mode: Section1286Mode::CompliantTaxExemptStrippedObligationOidProperlySplitPerSection1286C,
                        statutory_basis: "IRC § 1286(c)/(d) — tax-exempt stripped obligation OID split into tax-exempt portion (lower of coupon rate or stripped yield) and non-exempt portion".to_string(),
                        notes: format!(
                            "COMPLIANT: tax-exempt stripped obligation OID {} dollars properly split into tax-exempt portion (at lower of coupon rate or stripped obligation yield to maturity) and non-exempt portion (subject to ordinary OID inclusion); § 1286(c)/(d) requirements satisfied.",
                            computed_oid_dollars
                        ),
                        citations,
                        computed_oid_dollars,
                        de_minimis_threshold_dollars,
                    },
                    PurchaserAction::TaxExemptOidNotProperlySplitTaxExemptVsTaxablePortion => Output {
                        mode: Section1286Mode::ViolationTaxExemptStrippedObligationOidNotProperlySplitPostJune10_1987,
                        statutory_basis: "IRC § 1286(c)/(d) — tax-exempt stripped obligation OID must be split between tax-exempt and non-exempt portions per § 1879 of TRA 1986".to_string(),
                        notes: format!(
                            "VIOLATION: tax-exempt stripped obligation OID {} dollars NOT properly split between tax-exempt portion (limited to lower of coupon rate or stripped yield to maturity) and non-exempt portion subject to ordinary OID inclusion; § 1286(c)/(d) violated.",
                            computed_oid_dollars
                        ),
                        citations,
                        computed_oid_dollars,
                        de_minimis_threshold_dollars,
                    },
                    _ => Output {
                        mode: Section1286Mode::ViolationTaxExemptStrippedObligationOidNotProperlySplitPostJune10_1987,
                        statutory_basis: "IRC § 1286(c)/(d) — tax-exempt stripped obligation OID classification required".to_string(),
                        notes: "VIOLATION: tax-exempt stripped obligation requires § 1286(c)/(d) tax-exempt/non-exempt split treatment; purchaser_action did not assert proper split classification.".to_string(),
                        citations,
                        computed_oid_dollars,
                        de_minimis_threshold_dollars,
                    },
                };
            }
            match input.purchaser_action {
                PurchaserAction::PurchaserRecognizedOidInclusionPostJuly1_1982 => Output {
                    mode: Section1286Mode::CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982,
                    statutory_basis: "IRC § 1286(a) — purchaser of stripped bond/coupon recognizes OID equal to SRPM − ratable share of purchase price".to_string(),
                    notes: format!(
                        "COMPLIANT: purchaser of stripped bond/coupon recognized OID inclusion of {} dollars (= SRPM {} − ratable share of purchase price {}); de minimis threshold under § 1273(a)(3) is {} dollars; computed OID exceeds de minimis so § 1286(a) current OID inclusion required.",
                        computed_oid_dollars,
                        input.stated_redemption_price_at_maturity_dollars,
                        input.ratable_share_of_purchase_price_dollars,
                        de_minimis_threshold_dollars
                    ),
                    citations,
                    computed_oid_dollars,
                    de_minimis_threshold_dollars,
                },
                PurchaserAction::PurchaserDidNotRecognizeOid => Output {
                    mode: Section1286Mode::ViolationPurchaserFailedToRecognizeOidOnStrippedBondPostJuly1_1982,
                    statutory_basis: "IRC § 1286(a) — current OID inclusion required for stripped bonds/coupons purchased after July 1, 1982".to_string(),
                    notes: format!(
                        "VIOLATION: purchaser of stripped bond/coupon failed to recognize current OID inclusion of {} dollars; § 1286(a) requires treatment as bond originally issued on purchase date with OID equal to (SRPM − ratable share of purchase price).",
                        computed_oid_dollars
                    ),
                    citations,
                    computed_oid_dollars,
                    de_minimis_threshold_dollars,
                },
                _ => Output {
                    mode: Section1286Mode::ViolationPurchaserFailedToRecognizeOidOnStrippedBondPostJuly1_1982,
                    statutory_basis: "IRC § 1286(a) — purchaser action not asserted; OID inclusion required".to_string(),
                    notes: "VIOLATION: purchaser_action did not assert OID recognition; § 1286(a) treats failure to recognize as non-compliance.".to_string(),
                    citations,
                    computed_oid_dollars,
                    de_minimis_threshold_dollars,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_purchaser_input() -> Input {
        Input {
            taxpayer_role: TaxpayerRole::PurchaserOfStrippedBondOrCoupon,
            obligation_type: ObligationType::TaxableBond,
            purchase_date_status: PurchaseDateStatus::PurchasedAfterJuly1_1982PostTefraEffective,
            tax_exempt_amendment_date_status:
                TaxExemptAmendmentDateStatus::NotApplicableNotTaxExempt,
            stripper_action: StripperAction::NotApplicableNotStripper,
            purchaser_action: PurchaserAction::PurchaserRecognizedOidInclusionPostJuly1_1982,
            stated_redemption_price_at_maturity_dollars: 100_000,
            ratable_share_of_purchase_price_dollars: 60_000,
            years_to_maturity: 10,
        }
    }

    fn baseline_stripper_input() -> Input {
        Input {
            taxpayer_role: TaxpayerRole::StripperOfCouponsFromBond,
            obligation_type: ObligationType::TaxableBond,
            purchase_date_status: PurchaseDateStatus::PurchasedAfterJuly1_1982PostTefraEffective,
            tax_exempt_amendment_date_status:
                TaxExemptAmendmentDateStatus::NotApplicableNotTaxExempt,
            stripper_action:
                StripperAction::StripperIncludedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv,
            purchaser_action: PurchaserAction::NotApplicableNotPurchaser,
            stated_redemption_price_at_maturity_dollars: 100_000,
            ratable_share_of_purchase_price_dollars: 60_000,
            years_to_maturity: 10,
        }
    }

    #[test]
    fn obligation_not_stripped_not_applicable() {
        let mut input = baseline_purchaser_input();
        input.obligation_type = ObligationType::NotAStrippedBondOrCoupon;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::NotApplicableObligationIsNotAStrippedBondOrCoupon
        );
    }

    #[test]
    fn short_term_obligation_under_one_year_not_applicable() {
        let mut input = baseline_purchaser_input();
        input.obligation_type = ObligationType::ShortTermObligationLessThanOneYearAtIssue;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::NotApplicableShortTermObligationLessThanOneYearAtIssue
        );
    }

    #[test]
    fn purchased_before_july_1_1982_pre_tefra_not_applicable() {
        let mut input = baseline_purchaser_input();
        input.purchase_date_status =
            PurchaseDateStatus::PurchasedOnOrBeforeJuly1_1982PreTefraEffective;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::NotApplicableObligationPurchasedOnOrBeforeJuly1_1982PreTefraEffective
        );
    }

    #[test]
    fn tax_exempt_purchased_before_june_10_1987_pre_amendment_not_applicable() {
        let mut input = baseline_purchaser_input();
        input.obligation_type = ObligationType::TaxExemptObligation;
        input.tax_exempt_amendment_date_status =
            TaxExemptAmendmentDateStatus::PurchasedOnOrBeforeJune10_1987PreTraAmendmentEffective;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::NotApplicableTaxExemptStrippedObligationPurchasedOnOrBeforeJune10_1987PreAmendment
        );
    }

    #[test]
    fn not_involved_in_stripping_not_applicable() {
        let mut input = baseline_purchaser_input();
        input.taxpayer_role = TaxpayerRole::NotInvolvedInStrippingTransaction;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::NotApplicableNotInvolvedInStrippingTransaction
        );
    }

    #[test]
    fn purchaser_recognized_oid_compliant() {
        let output = check(&baseline_purchaser_input());
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982
        );
        // OID = 100_000 - 60_000 = 40_000
        assert_eq!(output.computed_oid_dollars, 40_000);
        // De minimis = 100_000 × 25 bps × 10 years / 10_000 = 2_500
        assert_eq!(output.de_minimis_threshold_dollars, 2_500);
    }

    #[test]
    fn purchaser_failed_to_recognize_oid_violation() {
        let mut input = baseline_purchaser_input();
        input.purchaser_action = PurchaserAction::PurchaserDidNotRecognizeOid;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::ViolationPurchaserFailedToRecognizeOidOnStrippedBondPostJuly1_1982
        );
    }

    #[test]
    fn stripper_compliant_with_full_basis_and_income_inclusion() {
        let output = check(&baseline_stripper_input());
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantStripperRecognizedAccruedInterestAndMarketDiscountAndAllocatedBasisByFmv
        );
    }

    #[test]
    fn stripper_failed_to_include_accrued_interest_violation() {
        let mut input = baseline_stripper_input();
        input.stripper_action = StripperAction::StripperDidNotIncludeAccruedInterestAndMarketDiscount;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::ViolationStripperFailedToIncludeAccruedInterestAndMarketDiscount
        );
    }

    #[test]
    fn stripper_failed_to_allocate_basis_by_fmv_violation() {
        let mut input = baseline_stripper_input();
        input.stripper_action = StripperAction::StripperDidNotAllocateBasisByFmv;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::ViolationStripperFailedToAllocateBasisByFairMarketValue
        );
    }

    #[test]
    fn tax_exempt_post_june_10_1987_properly_split_compliant() {
        let mut input = baseline_purchaser_input();
        input.obligation_type = ObligationType::TaxExemptObligation;
        input.tax_exempt_amendment_date_status =
            TaxExemptAmendmentDateStatus::PurchasedAfterJune10_1987PostTraAmendmentEffective;
        input.purchaser_action =
            PurchaserAction::TaxExemptOidProperlySplitTaxExemptAndTaxablePortions;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantTaxExemptStrippedObligationOidProperlySplitPerSection1286C
        );
    }

    #[test]
    fn tax_exempt_post_june_10_1987_not_properly_split_violation() {
        let mut input = baseline_purchaser_input();
        input.obligation_type = ObligationType::TaxExemptObligation;
        input.tax_exempt_amendment_date_status =
            TaxExemptAmendmentDateStatus::PurchasedAfterJune10_1987PostTraAmendmentEffective;
        input.purchaser_action =
            PurchaserAction::TaxExemptOidNotProperlySplitTaxExemptVsTaxablePortion;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::ViolationTaxExemptStrippedObligationOidNotProperlySplitPostJune10_1987
        );
    }

    #[test]
    fn de_minimis_oid_under_threshold_no_current_inclusion_required() {
        // SRPM 100_000 × 25 bps × 10 years / 10_000 = 2_500 threshold
        // OID = 100_000 - 99_000 = 1_000 < 2_500 → de minimis applies
        let mut input = baseline_purchaser_input();
        input.ratable_share_of_purchase_price_dollars = 99_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantOidUnderDeMinimisThresholdSection1273A3NoCurrentInclusionRequired
        );
        assert_eq!(output.computed_oid_dollars, 1_000);
        assert_eq!(output.de_minimis_threshold_dollars, 2_500);
    }

    #[test]
    fn de_minimis_threshold_boundary_at_exactly_threshold_not_de_minimis() {
        // OID = 2_500 = de_minimis threshold → strict less-than → NOT de minimis
        let mut input = baseline_purchaser_input();
        input.ratable_share_of_purchase_price_dollars = 97_500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982
        );
        assert_eq!(output.computed_oid_dollars, 2_500);
        assert_eq!(output.de_minimis_threshold_dollars, 2_500);
    }

    #[test]
    fn de_minimis_threshold_zero_when_zero_years_to_maturity() {
        let mut input = baseline_purchaser_input();
        input.years_to_maturity = 0;
        let output = check(&input);
        assert_eq!(output.de_minimis_threshold_dollars, 0);
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982
        );
    }

    #[test]
    fn oid_zero_when_purchase_equals_srpm_no_violation() {
        let mut input = baseline_purchaser_input();
        input.ratable_share_of_purchase_price_dollars = 100_000;
        let output = check(&input);
        assert_eq!(output.computed_oid_dollars, 0);
        // Zero OID: falls through past de minimis check (OID > 0 condition) into match branch
        assert_eq!(
            output.mode,
            Section1286Mode::CompliantPurchaserRecognizedOidOnStrippedBondPostJuly1_1982
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(IRC_1286_EFFECTIVE_DATE_YEAR, 1982);
        assert_eq!(IRC_1286_EFFECTIVE_DATE_MONTH, 7);
        assert_eq!(IRC_1286_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_YEAR, 1987);
        assert_eq!(IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_MONTH, 6);
        assert_eq!(IRC_1286_TAX_EXEMPT_AMENDMENT_EFFECTIVE_DATE_DAY, 10);
        assert_eq!(IRC_1273_A3_DE_MINIMIS_BASIS_POINTS_PER_YEAR, 25);
        assert_eq!(IRC_1286_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_section_1286_landmarks() {
        let output = check(&baseline_purchaser_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("§ 1286(a)"));
        assert!(joined.contains("§ 1286(b)"));
        assert!(joined.contains("§ 1286(c)"));
        assert!(joined.contains("§ 1286(d)"));
        assert!(joined.contains("§ 1273(a)(3)"));
        assert!(joined.contains("JULY 1, 1982"));
        assert!(joined.contains("JUNE 10, 1987"));
        assert!(joined.contains("§ 1232B"));
        assert!(joined.contains("Public Law 97-248"));
        assert!(joined.contains("Tax Reform Act of 1986"));
    }

    #[test]
    fn saturating_overflow_defense_extreme_inputs() {
        let mut input = baseline_purchaser_input();
        input.stated_redemption_price_at_maturity_dollars = u64::MAX;
        input.ratable_share_of_purchase_price_dollars = 0;
        input.years_to_maturity = u32::MAX;
        let output = check(&input);
        assert_eq!(output.computed_oid_dollars, u64::MAX);
        // De minimis computed via u128 saturating arithmetic, clamped to u64::MAX
        assert_eq!(output.de_minimis_threshold_dollars, u64::MAX);
    }
}
