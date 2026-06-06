//! IRC § 423 — Employee Stock Purchase Plans (ESPPs).
//! Direct companion to section_422 (ISOs — iter 438),
//! section_475c2 (mark-to-market for trader stock),
//! section_1411 (NIIT 3.8% on net investment income),
//! and the § 421(a) statutory option framework.
//!
//! Trader-critical because traders working at public
//! companies routinely participate in qualified ESPPs:
//! - **Qualified ESPP (§ 423)** — up to 15% DISCOUNT
//!   from fair market value with optional LOOK-BACK
//!   provision (purchase at lower of offering-date or
//!   exercise-date FMV); deferred ordinary income
//!   recognition (ISO-style); QUALIFYING DISPOSITION
//!   when held 2 years from grant + 1 year from
//!   purchase generates ordinary income capped at the
//!   discount with remainder as LTCG;
//! - **$25,000 annual accrual limit** under § 423(b)(8)
//!   measured by FMV at GRANT DATE (offering date),
//!   not discounted purchase price;
//! - **27-month outer offering limit** under § 423(b)(7)
//!   when plan uses look-back provision (5-year limit
//!   without look-back);
//! - **NO FICA** on qualified disposition ordinary
//!   income (Rev. Rul. 71-52 + Notice 2002-47 exempted
//!   ESPP from Social Security/Medicare withholding).
//!
//! **§ 423(b) ESPP STATUTORY REQUIREMENTS** — plan
//! must satisfy ALL nine conditions to qualify:
//! 1. § 423(b)(1) — plan provides options to EMPLOYEES
//!    of granting corporation, parent, or subsidiary;
//! 2. § 423(b)(2) — SHAREHOLDER-APPROVED within 12
//!    months before or after board adoption;
//! 3. § 423(b)(3) — NO 5%+ SHAREHOLDER may participate
//!    (constructive ownership rules under § 424(d));
//! 4. § 423(b)(4) — ALL EMPLOYEES eligible (with
//:    permitted exclusions for <2 year service, <20-
//!    hour part-time, and seasonal workers);
//! 5. § 423(b)(5) — SAME RIGHTS AND PRIVILEGES to all
//!    participants (with permitted compensation-based
//!    differentials);
//! 6. § 423(b)(6) — option price NOT LESS THAN LESSER
//!    of 85% of FMV at GRANT or 85% of FMV at EXERCISE
//!    (the look-back provision);
//! 7. § 423(b)(7) — OUTER OFFERING LIMIT 5 years (or
//!    27 months WITH look-back);
//! 8. § 423(b)(8) — $25,000 ANNUAL ACCRUAL CAP per
//!    calendar year (FMV at grant);
//! 9. § 423(b)(9) — NOT TRANSFERABLE except by will or
//!    inheritance.
//!
//! **§ 423(b)(6) LOOK-BACK PROVISION** — plan may set
//! purchase price at 85% of LOWER of:
//! - FMV at OFFERING (grant) date;
//! - FMV at PURCHASE (exercise) date.
//!
//! Combined with intra-period appreciation, can yield
//! effective discount substantially greater than 15%.
//!
//! **§ 421(a) QUALIFYING DISPOSITION HOLDING PERIODS**:
//! 1. 2 YEARS from OFFERING (grant) date;
//! 2. 1 YEAR from PURCHASE (exercise) date;
//! 3. BOTH must be satisfied — fails either =
//!    DISQUALIFYING DISPOSITION.
//!
//! **§ 423(c) QUALIFYING DISPOSITION TREATMENT** —
//! when ESPP shares held through both holding periods:
//! - ORDINARY INCOME = LESSER of (a) actual discount
//!   in option price (15% × FMV at offering); OR (b)
//!   actual gain on sale (sale price − purchase price);
//! - REMAINING GAIN = LONG-TERM CAPITAL GAIN taxed at
//!   0%/15%/20% + § 1411 NIIT 3.8%;
//! - NO FICA WITHHOLDING on the discount portion
//!   (Notice 2002-47 + Rev. Rul. 71-52);
//! - Employer GETS NO § 162 compensation deduction
//!   for qualifying disposition.
//!
//! **§ 421(b) DISQUALIFYING DISPOSITION TREATMENT** —
//! shares sold before holding periods satisfied:
//! - ORDINARY INCOME = FULL SPREAD AT PURCHASE (FMV
//!   at purchase date − actual purchase price); NOT
//!   capped at discount;
//! - REMAINING GAIN = short-term or long-term capital
//!   gain (or loss) depending on holding from purchase;
//! - Employer GETS § 162 compensation deduction equal
//!   to ordinary-income amount.
//!
//! **§ 423(b)(8) $25,000 ANNUAL ACCRUAL CAP** — right
//! to purchase under all § 423 plans of employer
//! cannot accrue at rate exceeding $25,000 of stock
//! value (FMV at GRANT DATE, not discount price) per
//! calendar year for any single offering period.
//! Carryover unused capacity to next year NOT allowed.
//!
//! **Trader-critical fact patterns**:
//! 1. Trader contributes $25K to ESPP with 15%
//!    look-back; stock $100 → $130 over 6 months;
//!    purchases at $85 (look-back from $100 grant
//!    date) = 250 shares at effective 34.6% discount
//!    ($85 vs $130 FMV); qualifying disposition at
//!    $150 = $15/share ordinary income (capped at
//!    discount) + $50/share LTCG = $3,750 ordinary +
//!    $12,500 LTCG;
//! 2. Same trader sells immediately at $130
//!    (disqualifying) — $45/share FULL SPREAD as
//!    ordinary income = $11,250 ordinary; no LTCG;
//! 3. § 423(b)(8) $25K cap — trader earning $300K
//!    cannot purchase more than ~$294 of stock
//!    (FMV at grant) per year under any single ESPP
//!    cycle if 15% discount applies — limit is on
//!    FMV at GRANT, not contribution;
//! 4. § 423(b)(3) — trader who is 5%+ shareholder of
//!    employer CANNOT participate under constructive
//!    ownership rules of § 424(d);
//! 5. § 423(b)(7) — plan with look-back limited to
//!    27-month offering period; plan WITHOUT
//!    look-back limited to 5-year offering period.
//!
//! Citations: 26 USC § 423(a)-(c); 26 USC § 421(a)-(b);
//! 26 USC § 424(d) (constructive ownership); 26 USC
//! § 1411 (NIIT 3.8%); 26 USC § 162 (employer
//! deduction on disqualifying disposition); Treas. Reg.
//! § 1.423-1 through § 1.423-2; Treas. Reg. § 1.421-1
//! through § 1.421-2; IRS Notice 2002-47 (FICA
//! exemption); Rev. Rul. 71-52 (no FICA on qualifying
//! disposition); Form 3922 (ESPP Transfer Information
//! Statement); IRS Pub. 525.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    NoDisposition,
    QualifyingDisposition,
    DisqualifyingDisposition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section423Input {
    /// FMV at offering (grant) date in cents per share.
    pub fmv_at_offering_cents: u64,
    /// FMV at purchase (exercise) date in cents per
    /// share.
    pub fmv_at_purchase_cents: u64,
    /// Actual purchase price per share in cents.
    pub purchase_price_cents: u64,
    /// Sale price per share in cents (0 if no
    /// disposition).
    pub sale_price_cents: u64,
    pub shares_count: u64,
    pub disposition: DispositionType,
    /// Whether plan uses § 423(b)(6) look-back
    /// provision.
    pub look_back_provision: bool,
    /// Discount percentage (e.g. 15 for 15%); plan
    /// must offer ≥ 85% of FMV (≤ 15% discount).
    pub discount_percent: u32,
    /// Annual accrual at grant FMV in dollars (§ 423
    /// (b)(8) $25K cap).
    pub annual_accrual_grant_fmv_dollars: u64,
    /// Whether plan shareholder-approved within 12
    /// months of board adoption (§ 423(b)(2)).
    pub shareholder_approved_within_12_months: bool,
    /// Whether participant is 5%+ shareholder (§ 423
    /// (b)(3) — disqualified).
    pub five_percent_shareholder: bool,
    /// Whether offering period within § 423(b)(7) outer
    /// limit (27 months with look-back / 5 years
    /// without).
    pub offering_period_within_outer_limit: bool,
    /// Whether plan grants same rights and privileges
    /// to all participants (§ 423(b)(5)).
    pub same_rights_and_privileges: bool,
    /// Years between offering date and sale.
    pub years_offering_to_sale: u32,
    /// Years between purchase date and sale.
    pub years_purchase_to_sale: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section423Result {
    pub plan_qualifies_under_section_423_b: bool,
    pub effective_purchase_price_cents: u64,
    pub effective_discount_cents_per_share: u64,
    pub annual_accrual_cap_exceeded: bool,
    pub holding_periods_satisfied: bool,
    pub ordinary_income_qualifying_disposition_cents: u64,
    pub ordinary_income_disqualifying_disposition_cents: u64,
    pub long_term_capital_gain_cents: u64,
    pub fica_exempt: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section423Input) -> Section423Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let plan_statutory_compliant = input.shareholder_approved_within_12_months
        && !input.five_percent_shareholder
        && input.offering_period_within_outer_limit
        && input.same_rights_and_privileges
        && input.discount_percent <= 15;

    let plan_qualifies_under_section_423_b = plan_statutory_compliant;

    let lower_fmv = if input.look_back_provision {
        input.fmv_at_offering_cents.min(input.fmv_at_purchase_cents)
    } else {
        input.fmv_at_purchase_cents
    };
    let effective_purchase_price_cents =
        lower_fmv.saturating_mul(100u64.saturating_sub(input.discount_percent as u64)) / 100;

    let effective_discount_cents_per_share = input
        .fmv_at_purchase_cents
        .saturating_sub(input.purchase_price_cents);

    let annual_accrual_cap_exceeded = input.annual_accrual_grant_fmv_dollars > 25_000;

    let holding_periods_satisfied = input.years_offering_to_sale >= 2
        && input.years_purchase_to_sale >= 1
        && matches!(input.disposition, DispositionType::QualifyingDisposition);

    let discount_at_offering_cents = input
        .fmv_at_offering_cents
        .saturating_mul(input.discount_percent as u64)
        / 100;
    let actual_gain_per_share = input
        .sale_price_cents
        .saturating_sub(input.purchase_price_cents);
    let lesser_qualifying = discount_at_offering_cents.min(actual_gain_per_share);

    let ordinary_income_qualifying_disposition_cents = if holding_periods_satisfied {
        lesser_qualifying.saturating_mul(input.shares_count)
    } else {
        0
    };

    let full_spread_at_purchase = input
        .fmv_at_purchase_cents
        .saturating_sub(input.purchase_price_cents);
    let ordinary_income_disqualifying_disposition_cents =
        if matches!(input.disposition, DispositionType::DisqualifyingDisposition) {
            full_spread_at_purchase.saturating_mul(input.shares_count)
        } else {
            0
        };

    let long_term_capital_gain_cents = if holding_periods_satisfied {
        let total_gain = input
            .sale_price_cents
            .saturating_sub(input.purchase_price_cents)
            .saturating_mul(input.shares_count);
        total_gain.saturating_sub(ordinary_income_qualifying_disposition_cents)
    } else {
        0
    };

    let fica_exempt = holding_periods_satisfied;

    if !input.shareholder_approved_within_12_months {
        failure_reasons.push(
            "26 USC § 423(b)(2) — plan NOT SHAREHOLDER-APPROVED within 12 months before or after board adoption; FAILS § 423 statutory requirement; treated as non-qualified ESPP (NQSO-style ordinary income on purchase)".to_string(),
        );
    }
    if input.five_percent_shareholder {
        failure_reasons.push(
            "26 USC § 423(b)(3) — participant is 5%+ SHAREHOLDER of employer (under § 424(d) constructive ownership rules); INELIGIBLE for qualified ESPP treatment".to_string(),
        );
    }
    if !input.offering_period_within_outer_limit {
        failure_reasons.push(
            "26 USC § 423(b)(7) — offering period EXCEEDS outer limit (27 months WITH look-back / 5 years WITHOUT look-back); FAILS § 423 statutory requirement".to_string(),
        );
    }
    if !input.same_rights_and_privileges {
        failure_reasons.push(
            "26 USC § 423(b)(5) — plan must provide SAME RIGHTS AND PRIVILEGES to all participants (with permitted compensation-based differentials); FAILS § 423 statutory requirement".to_string(),
        );
    }
    if input.discount_percent > 15 {
        failure_reasons.push(format!(
            "26 USC § 423(b)(6) — discount {}% EXCEEDS 15% maximum; option price must be NOT LESS THAN 85% of FMV at lesser of offering or exercise date; FAILS § 423 statutory requirement",
            input.discount_percent
        ));
    }

    if annual_accrual_cap_exceeded {
        failure_reasons.push(format!(
            "26 USC § 423(b)(8) — annual accrual ${} EXCEEDS $25,000 cap (measured by FMV at GRANT DATE, not discount price); excess INELIGIBLE for § 423 treatment; ordinary income on excess on purchase",
            input.annual_accrual_grant_fmv_dollars
        ));
    }

    if input.look_back_provision {
        failure_reasons.push(format!(
            "26 USC § 423(b)(6) LOOK-BACK PROVISION — effective purchase price {} cents/share (85% of LOWER of grant FMV {} and purchase FMV {}); effective discount {} cents/share due to intra-period appreciation can substantially exceed nominal 15%",
            effective_purchase_price_cents,
            input.fmv_at_offering_cents,
            input.fmv_at_purchase_cents,
            effective_discount_cents_per_share
        ));
    }

    if holding_periods_satisfied {
        failure_reasons.push(format!(
            "26 USC § 421(a) + § 423(c) QUALIFYING DISPOSITION — 2-year-from-offering + 1-year-from-purchase periods SATISFIED; ordinary income {} cents (LESSER of 15% discount at offering OR actual gain); LTCG {} cents at 0%/15%/20% + § 1411 NIIT 3.8%; NO FICA on ordinary income portion (Notice 2002-47 + Rev. Rul. 71-52)",
            ordinary_income_qualifying_disposition_cents,
            long_term_capital_gain_cents
        ));
    }

    if matches!(input.disposition, DispositionType::DisqualifyingDisposition) {
        failure_reasons.push(format!(
            "26 USC § 421(b) DISQUALIFYING DISPOSITION — fails § 421(a) 2-year-from-offering + 1-year-from-purchase holding periods; FULL SPREAD at purchase ({} cents/share = FMV purchase - actual price) recharacterized as ORDINARY COMPENSATION INCOME up to 37%; total ordinary income {} cents; employer GETS § 162 compensation deduction",
            full_spread_at_purchase,
            ordinary_income_disqualifying_disposition_cents
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 423(b) ESPP STATUTORY REQUIREMENTS (9 conditions): (1) options to employees of granting corp/parent/sub; (2) shareholder-approved within 12 months; (3) no 5%+ shareholder under § 424(d); (4) all employees eligible (permitted < 2 year, < 20 hour, seasonal exclusions); (5) same rights and privileges (permitted compensation differentials); (6) price ≥ 85% of lesser of offering or exercise FMV; (7) outer limit 5 years (27 months with look-back); (8) $25K annual accrual cap; (9) not transferable except by will/inheritance".to_string(),
        "26 USC § 423(b)(6) LOOK-BACK PROVISION — plan may set purchase price at 85% of LOWER of FMV at offering (grant) date OR FMV at purchase (exercise) date; combined with intra-period appreciation effective discount can substantially exceed 15%".to_string(),
        "26 USC § 423(b)(8) $25,000 ANNUAL ACCRUAL CAP — right to purchase under all § 423 plans cannot accrue at rate exceeding $25,000 of FMV at GRANT DATE (not discount price) per calendar year for any single offering period; unused capacity NOT carried over".to_string(),
        "26 USC § 421(a) QUALIFYING DISPOSITION HOLDING PERIODS — (1) 2 YEARS from OFFERING (grant) date; (2) 1 YEAR from PURCHASE (exercise) date; BOTH must be satisfied".to_string(),
        "26 USC § 423(c) QUALIFYING DISPOSITION TREATMENT — ordinary income = LESSER of (a) discount at offering date (15% × FMV at offering) OR (b) actual gain (sale − purchase); remaining gain = LONG-TERM CAPITAL GAIN at 0/15/20% + § 1411 NIIT 3.8%; NO FICA on ordinary income; employer gets NO § 162 deduction".to_string(),
        "26 USC § 421(b) DISQUALIFYING DISPOSITION TREATMENT — ordinary income = FULL SPREAD at PURCHASE (FMV at purchase − actual price); not capped at discount; remaining gain = ST or LT capital gain depending on holding from purchase; employer GETS § 162 compensation deduction".to_string(),
        "IRS Notice 2002-47 + Rev. Rul. 71-52 — FICA EXEMPTION on qualifying disposition; ordinary income recognized on QUALIFYING DISPOSITION (the discount portion) is EXEMPT from Social Security/Medicare withholding; DISQUALIFYING DISPOSITION ordinary income IS subject to FICA withholding".to_string(),
        "26 USC § 424(d) CONSTRUCTIVE OWNERSHIP — 5%+ shareholder disqualification under § 423(b)(3) includes shares owned directly + shares owned by family (spouse, ancestors, lineal descendants) + proportionate share of partnership/corporation/estate/trust holdings".to_string(),
        "26 USC § 1411 NIIT 3.8% — qualifying disposition LTCG INCLUDED in net investment income; 3.8% NIIT applies above MAGI thresholds ($200K single / $250K MFJ); disqualifying disposition ORDINARY INCOME portion EXCLUDED from NII (compensation/wages excluded)".to_string(),
        "Trader-critical fact patterns: (1) $25K contributions; 15% look-back; stock $100 → $130 over 6 months → purchases at $85; qualifying disposition at $150 = $15/share ordinary (capped at discount) + $50/share LTCG; (2) immediate disqualifying disposition = $45/share FULL SPREAD ordinary income; (3) $25K cap on FMV-at-GRANT (not contribution); (4) § 423(b)(3) 5%+ shareholder cannot participate; (5) § 423(b)(7) 27-month limit with look-back / 5-year without".to_string(),
        "Form 3922 — ESPP Transfer Information Statement; employer must furnish to employee by January 31 of year after FIRST TRANSFER of legal title; reports offering date, exercise date, FMVs, purchase price, shares acquired; trader must keep for life of stock to track qualifying-vs-disqualifying basis split".to_string(),
        "Companion to section_422 (ISO — alternative qualified-stock-option regime; § 422 requires 100% strike price; § 423 allows 15% discount), section_475c2 (mark-to-market for trader stock; ESPP basis split unaffected by election), section_1411 (NIIT 3.8% on qualifying LTCG), section_424 (constructive ownership rules), section_421 (general statutory option framework)".to_string(),
    ];

    Section423Result {
        plan_qualifies_under_section_423_b,
        effective_purchase_price_cents,
        effective_discount_cents_per_share,
        annual_accrual_cap_exceeded,
        holding_periods_satisfied,
        ordinary_income_qualifying_disposition_cents,
        ordinary_income_disqualifying_disposition_cents,
        long_term_capital_gain_cents,
        fica_exempt,
        failure_reasons,
        citation: "26 USC § 423(a)-(c); 26 USC § 421(a)-(b); 26 USC § 424(d) (constructive ownership); 26 USC § 1411 (NIIT 3.8%); 26 USC § 162 (employer deduction); Treas. Reg. § 1.423-1 through § 1.423-2; Treas. Reg. § 1.421-1 through § 1.421-2; IRS Notice 2002-47 (FICA exemption); Rev. Rul. 71-52 (no FICA on qualifying disposition); Form 3922 (ESPP Transfer Information Statement); IRS Pub. 525",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn espp_qualifying_lookback() -> Section423Input {
        Section423Input {
            fmv_at_offering_cents: 10_000,
            fmv_at_purchase_cents: 13_000,
            purchase_price_cents: 8_500,
            sale_price_cents: 15_000,
            shares_count: 250,
            disposition: DispositionType::QualifyingDisposition,
            look_back_provision: true,
            discount_percent: 15,
            annual_accrual_grant_fmv_dollars: 25_000,
            shareholder_approved_within_12_months: true,
            five_percent_shareholder: false,
            offering_period_within_outer_limit: true,
            same_rights_and_privileges: true,
            years_offering_to_sale: 3,
            years_purchase_to_sale: 2,
        }
    }

    #[test]
    fn qualifying_disposition_lesser_of_rule() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.plan_qualifies_under_section_423_b);
        assert!(r.holding_periods_satisfied);
        assert_eq!(r.ordinary_income_qualifying_disposition_cents, 1500 * 250);
    }

    #[test]
    fn qualifying_disposition_ltcg_remainder() {
        let r = check(&espp_qualifying_lookback());
        let total_gain = (15_000 - 8_500) * 250;
        let ord = 1500 * 250;
        assert_eq!(r.long_term_capital_gain_cents, total_gain - ord);
    }

    #[test]
    fn qualifying_disposition_fica_exempt() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.fica_exempt);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Notice 2002-47") && f.contains("NO FICA")));
    }

    #[test]
    fn look_back_effective_price() {
        let r = check(&espp_qualifying_lookback());
        assert_eq!(r.effective_purchase_price_cents, 8_500);
        assert_eq!(r.effective_discount_cents_per_share, 13_000 - 8_500);
    }

    #[test]
    fn no_lookback_uses_purchase_fmv() {
        let mut i = espp_qualifying_lookback();
        i.look_back_provision = false;
        let r = check(&i);
        assert_eq!(r.effective_purchase_price_cents, 13_000 * 85 / 100);
    }

    #[test]
    fn disqualifying_disposition_full_spread_ordinary() {
        let mut i = espp_qualifying_lookback();
        i.disposition = DispositionType::DisqualifyingDisposition;
        i.years_offering_to_sale = 1;
        i.years_purchase_to_sale = 0;
        i.sale_price_cents = 13_000;
        let r = check(&i);
        assert!(!r.holding_periods_satisfied);
        assert_eq!(
            r.ordinary_income_disqualifying_disposition_cents,
            (13_000 - 8_500) * 250
        );
        assert!(
            r.failure_reasons
                .iter()
                .any(|f| f.contains("§ 421(b) DISQUALIFYING DISPOSITION")
                    && f.contains("FULL SPREAD"))
        );
    }

    #[test]
    fn disqualifying_no_fica_exempt() {
        let mut i = espp_qualifying_lookback();
        i.disposition = DispositionType::DisqualifyingDisposition;
        i.years_offering_to_sale = 1;
        i.years_purchase_to_sale = 0;
        let r = check(&i);
        assert!(!r.fica_exempt);
    }

    #[test]
    fn no_shareholder_approval_fails() {
        let mut i = espp_qualifying_lookback();
        i.shareholder_approved_within_12_months = false;
        let r = check(&i);
        assert!(!r.plan_qualifies_under_section_423_b);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 423(b)(2)") && f.contains("12 months")));
    }

    #[test]
    fn five_percent_shareholder_disqualified() {
        let mut i = espp_qualifying_lookback();
        i.five_percent_shareholder = true;
        let r = check(&i);
        assert!(!r.plan_qualifies_under_section_423_b);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 423(b)(3)")
            && f.contains("5%+ SHAREHOLDER")
            && f.contains("§ 424(d)")));
    }

    #[test]
    fn offering_period_over_outer_limit_fails() {
        let mut i = espp_qualifying_lookback();
        i.offering_period_within_outer_limit = false;
        let r = check(&i);
        assert!(!r.plan_qualifies_under_section_423_b);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 423(b)(7)") && f.contains("27 months WITH look-back")));
    }

    #[test]
    fn unequal_rights_privileges_fails() {
        let mut i = espp_qualifying_lookback();
        i.same_rights_and_privileges = false;
        let r = check(&i);
        assert!(!r.plan_qualifies_under_section_423_b);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 423(b)(5)") && f.contains("SAME RIGHTS AND PRIVILEGES")));
    }

    #[test]
    fn discount_over_15_percent_fails() {
        let mut i = espp_qualifying_lookback();
        i.discount_percent = 20;
        let r = check(&i);
        assert!(!r.plan_qualifies_under_section_423_b);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 423(b)(6)") && f.contains("EXCEEDS 15% maximum")));
    }

    #[test]
    fn annual_accrual_over_25k_exceeded() {
        let mut i = espp_qualifying_lookback();
        i.annual_accrual_grant_fmv_dollars = 30_000;
        let r = check(&i);
        assert!(r.annual_accrual_cap_exceeded);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 423(b)(8)")
            && f.contains("$25,000 cap")
            && f.contains("GRANT DATE")));
    }

    #[test]
    fn annual_accrual_at_25k_boundary() {
        let mut i = espp_qualifying_lookback();
        i.annual_accrual_grant_fmv_dollars = 25_000;
        let r = check(&i);
        assert!(!r.annual_accrual_cap_exceeded);
    }

    #[test]
    fn holding_periods_truth_table() {
        // 2+1 satisfied
        let r1 = check(&espp_qualifying_lookback());
        assert!(r1.holding_periods_satisfied);

        // 1 year from offering — fails
        let mut a = espp_qualifying_lookback();
        a.years_offering_to_sale = 1;
        a.disposition = DispositionType::QualifyingDisposition;
        assert!(!check(&a).holding_periods_satisfied);

        // 0 years from purchase — fails
        let mut b = espp_qualifying_lookback();
        b.years_purchase_to_sale = 0;
        b.disposition = DispositionType::QualifyingDisposition;
        assert!(!check(&b).holding_periods_satisfied);
    }

    #[test]
    fn no_disposition_no_income_recognition() {
        let mut i = espp_qualifying_lookback();
        i.disposition = DispositionType::NoDisposition;
        i.sale_price_cents = 0;
        let r = check(&i);
        assert_eq!(r.ordinary_income_qualifying_disposition_cents, 0);
        assert_eq!(r.ordinary_income_disqualifying_disposition_cents, 0);
        assert_eq!(r.long_term_capital_gain_cents, 0);
    }

    #[test]
    fn disposition_truth_table_three_cells() {
        for (disp, expect_qd, expect_dq) in [
            (DispositionType::NoDisposition, false, false),
            (DispositionType::QualifyingDisposition, true, false),
            (DispositionType::DisqualifyingDisposition, false, true),
        ] {
            let mut i = espp_qualifying_lookback();
            i.disposition = disp;
            if matches!(disp, DispositionType::DisqualifyingDisposition) {
                i.years_offering_to_sale = 1;
                i.years_purchase_to_sale = 0;
            }
            let r = check(&i);
            assert_eq!(
                r.ordinary_income_qualifying_disposition_cents > 0,
                expect_qd,
                "disp={:?}",
                disp
            );
            assert_eq!(
                r.ordinary_income_disqualifying_disposition_cents > 0,
                expect_dq,
                "disp={:?}",
                disp
            );
        }
    }

    #[test]
    fn qualifying_uniquely_fica_exempt_invariant() {
        let mut qd = espp_qualifying_lookback();
        qd.disposition = DispositionType::QualifyingDisposition;
        let r_qd = check(&qd);
        assert!(r_qd.fica_exempt);

        let mut dq = espp_qualifying_lookback();
        dq.disposition = DispositionType::DisqualifyingDisposition;
        dq.years_offering_to_sale = 1;
        dq.years_purchase_to_sale = 0;
        let r_dq = check(&dq);
        assert!(!r_dq.fica_exempt);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.citation.contains("§ 423(a)-(c)"));
        assert!(r.citation.contains("§ 421(a)-(b)"));
        assert!(r.citation.contains("§ 424(d)"));
        assert!(r.citation.contains("§ 1411 (NIIT 3.8%)"));
        assert!(r.citation.contains("§ 162 (employer deduction)"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.423-1 through § 1.423-2"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.421-1 through § 1.421-2"));
        assert!(r.citation.contains("IRS Notice 2002-47"));
        assert!(r.citation.contains("Rev. Rul. 71-52"));
        assert!(r.citation.contains("Form 3922"));
    }

    #[test]
    fn note_pins_subsection_b_nine_requirements() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n.contains("§ 423(b)")
            && n.contains("(9 conditions)")
            && n.contains("§ 424(d)")
            && n.contains("$25K annual accrual")));
    }

    #[test]
    fn note_pins_subsection_b6_look_back() {
        let r = check(&espp_qualifying_lookback());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 423(b)(6) LOOK-BACK PROVISION")
                && n.contains("85% of LOWER")
                && n.contains("substantially exceed 15%")));
    }

    #[test]
    fn note_pins_subsection_b8_25k_cap() {
        let r = check(&espp_qualifying_lookback());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 423(b)(8) $25,000 ANNUAL ACCRUAL CAP")
                && n.contains("GRANT DATE")
                && n.contains("NOT carried over")));
    }

    #[test]
    fn note_pins_subsection_421a_holding_periods() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 421(a) QUALIFYING DISPOSITION HOLDING PERIODS")
            && n.contains("2 YEARS from OFFERING")
            && n.contains("1 YEAR from PURCHASE")));
    }

    #[test]
    fn note_pins_subsection_423c_qualifying_treatment() {
        let r = check(&espp_qualifying_lookback());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 423(c) QUALIFYING DISPOSITION TREATMENT")
                && n.contains("LESSER of")
                && n.contains("NO FICA")
                && n.contains("NO § 162 deduction")));
    }

    #[test]
    fn note_pins_subsection_421b_disqualifying_treatment() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 421(b) DISQUALIFYING DISPOSITION TREATMENT")
                && n.contains("FULL SPREAD")
                && n.contains("GETS § 162 compensation deduction")
        ));
    }

    #[test]
    fn note_pins_notice_2002_47_fica() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n.contains("Notice 2002-47")
            && n.contains("Rev. Rul. 71-52")
            && n.contains("FICA EXEMPTION")));
    }

    #[test]
    fn note_pins_section_424d_constructive_ownership() {
        let r = check(&espp_qualifying_lookback());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 424(d) CONSTRUCTIVE OWNERSHIP")
                && n.contains("family")
                && n.contains("partnership/corporation/estate/trust")));
    }

    #[test]
    fn note_pins_section_1411_niit() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n.contains("§ 1411 NIIT 3.8%")
            && n.contains("$200K single")
            && n.contains("$250K MFJ")));
    }

    #[test]
    fn note_pins_trader_fact_patterns() {
        let r = check(&espp_qualifying_lookback());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("$25K contributions")
                && n.contains("look-back")
                && n.contains("$45/share FULL SPREAD")));
    }

    #[test]
    fn note_pins_form_3922() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n.contains("Form 3922")
            && n.contains("FIRST TRANSFER")
            && n.contains("January 31")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&espp_qualifying_lookback());
        assert!(r.notes.iter().any(|n| n.contains("section_422")
            && n.contains("section_475c2")
            && n.contains("section_1411")
            && n.contains("section_424")
            && n.contains("section_421")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = espp_qualifying_lookback();
        i.fmv_at_purchase_cents = u64::MAX;
        i.shares_count = u64::MAX;
        let r = check(&i);
        let _ = r.ordinary_income_qualifying_disposition_cents;
    }
}
