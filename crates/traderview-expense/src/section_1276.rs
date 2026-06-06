//! IRC § 1276 — Disposition gain representing accrued market
//! discount treated as ordinary income.
//!
//! Trader-critical bond rule. Every secondary-market bond purchased
//! at a price below stated redemption (face value) at maturity has
//! "market discount" under § 1278(a)(2)(A). On disposition (sale,
//! exchange, partial principal payment, or non-sale event treated
//! under § 1276(a)(2)), gain is converted from capital to ORDINARY
//! INCOME to the extent of accrued market discount.
//!
//! Companion to:
//!   - § 1277 (deferral of interest deduction on indebtedness
//!     incurred to purchase or carry market discount bonds).
//!   - § 1278 (definitions; § 1278(b) current-inclusion election).
//!   - § 1272 (current inclusion of OID — separate but cross-
//!     referenced by § 1276(b)(2) constant-yield computation).
//!
//! Five operative subparagraphs of § 1276(a) and two accrual
//! methods under § 1276(b):
//!
//!   § 1276(a)(1) — GENERAL RULE: Gain on disposition of any market
//!     discount bond is ordinary income up to accrued market
//!     discount.
//!
//!   § 1276(a)(2) — DISPOSITIONS OTHER THAN SALES: Non-sale
//!     dispositions (gift, etc.) treated as realizing an amount
//!     equal to FMV of the bond.
//!
//!   § 1276(a)(3) — PARTIAL PRINCIPAL PAYMENTS: A partial principal
//!     payment is ordinary income up to accrued market discount.
//!
//!   § 1276(a)(4) — TREATMENT AS INTEREST: Any amount treated as
//!     ordinary income under § 1276(a)(1) or (3) is treated as
//!     INTEREST for purposes of the Code (with carve-outs for
//!     §§ 103, 871(a), 881, 1441, 1442, 6049).
//!
//!   § 1276(b)(1) — RATABLE ACCRUAL (DEFAULT): Accrued discount =
//!     market discount × (days held ÷ days from acquisition to
//!     maturity).
//!
//!   § 1276(b)(2) — CONSTANT-YIELD ELECTION: Taxpayer may elect to
//!     compute accrued market discount under the § 1272(a) OID
//!     yield-to-maturity formula as if bond were originally issued
//!     on acquisition date at the taxpayer's basis. Election is
//!     irrevocable as to that bond.
//!
//! § 1278(b) current-inclusion election permits a taxpayer to
//! INCLUDE accrued market discount in income each year rather than
//! deferring to disposition. If elected, accrued discount already
//! taxed in prior years is excluded from § 1276 ordinary-income
//! recharacterization on disposition.
//!
//! Citations: 26 U.S.C. § 1276(a)(1) (general ordinary-income
//! rule); § 1276(a)(2) (non-sale dispositions); § 1276(a)(3)
//! (partial principal payments); § 1276(a)(4) (treatment as
//! interest); § 1276(b)(1) (ratable accrual default); § 1276(b)(2)
//! (constant-yield election); § 1278(a)(2)(A) (market discount
//! definition); § 1278(b) (current-inclusion election); § 1272(a)
//! (OID accrual rule referenced by § 1276(b)(2)).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccrualMethod {
    /// § 1276(b)(1) — default ratable accrual proportional to days
    /// held over total days from acquisition to maturity.
    RatableAccrual,
    /// § 1276(b)(2) — election to use § 1272(a) OID yield-to-
    /// maturity formula. Caller supplies the computed accrual
    /// in `constant_yield_accrual_cents`.
    ConstantYield,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    /// § 1276(a)(1) — sale, exchange, or involuntary conversion.
    Sale,
    /// § 1276(a)(3) — partial principal payment on the bond.
    PartialPrincipalPayment,
    /// § 1276(a)(2) — non-sale disposition (gift, distribution,
    /// etc.); realized amount = FMV of bond.
    NonSaleDisposition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1276Input {
    /// Taxpayer's purchase price (basis at acquisition) in cents.
    /// Used to compute market discount = stated redemption − basis.
    pub purchase_price_cents: i64,
    /// Stated redemption price at maturity (face value) in cents.
    pub stated_redemption_at_maturity_cents: i64,
    /// Total days from acquisition date to bond maturity date.
    /// Used as the denominator under § 1276(b)(1) ratable accrual.
    pub days_from_acquisition_to_maturity: u32,
    /// Days the taxpayer has held the bond at the time of
    /// disposition. Numerator under § 1276(b)(1) ratable accrual.
    /// Capped at `days_from_acquisition_to_maturity`.
    pub days_held: u32,
    pub disposition_type: DispositionType,
    /// Realized amount on disposition in cents. For Sale this is
    /// the sale price; for PartialPrincipalPayment, the principal
    /// payment amount; for NonSaleDisposition, the FMV of the bond
    /// (which § 1276(a)(2) deems realized).
    pub realized_amount_cents: i64,
    /// § 1276(b) accrual method to apply.
    pub accrual_method: AccrualMethod,
    /// § 1276(b)(2) constant-yield-method accrued amount supplied
    /// by caller (computed under § 1272(a) OID formula). Used only
    /// when `accrual_method = ConstantYield`.
    pub constant_yield_accrual_cents: i64,
    /// Whether the taxpayer elected current inclusion of accrued
    /// market discount under § 1278(b) for this bond.
    pub current_inclusion_election: bool,
    /// Cumulative accrued market discount already taxed as
    /// ordinary income in prior tax years under § 1278(b) current
    /// inclusion. Subtracted from the § 1276 ordinary cap so the
    /// same accrual is not taxed twice.
    pub prior_years_accrual_already_taxed_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1276Result {
    /// § 1278(a)(2)(A) market discount = stated redemption − basis
    /// (clamped at zero — no negative discount).
    pub market_discount_cents: i64,
    /// Accrued market discount under the chosen method (cents).
    pub accrued_market_discount_cents: i64,
    /// Amount of gain recharacterized as ordinary income under
    /// § 1276 (cents). Capped at accrued market discount net of
    /// prior-year inclusions.
    pub ordinary_income_cents: i64,
    /// Residual capital portion of gain (cents).
    pub capital_gain_cents: i64,
    /// § 1276(a)(4) — true when § 1276 ordinary recharacterization
    /// applies (so the amount is treated as INTEREST for Code
    /// purposes other than §§ 103, 871(a), 881, 1441, 1442, 6049).
    pub treated_as_interest: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1276Input) -> Section1276Result {
    let mut notes: Vec<String> = Vec::new();

    // § 1278(a)(2)(A) — market discount = stated redemption − basis
    // (clamped at zero).
    let market_discount = input
        .stated_redemption_at_maturity_cents
        .saturating_sub(input.purchase_price_cents)
        .max(0);

    if market_discount == 0 {
        notes.push(
            "§ 1278(a)(2)(A) — purchase price is at or above stated redemption at maturity; \
             there is no market discount and § 1276 does not apply."
                .to_string(),
        );
        let total_gain = input
            .realized_amount_cents
            .saturating_sub(input.purchase_price_cents);
        return Section1276Result {
            market_discount_cents: 0,
            accrued_market_discount_cents: 0,
            ordinary_income_cents: 0,
            capital_gain_cents: total_gain.max(0),
            treated_as_interest: false,
            citation: "26 U.S.C. § 1278(a)(2)(A) (market discount definition — no discount \
                       where basis ≥ stated redemption at maturity); § 1276 inapplicable",
            notes,
        };
    }

    // § 1276(b) — compute accrued market discount under the chosen
    // method.
    let accrued = match input.accrual_method {
        AccrualMethod::RatableAccrual => {
            // § 1276(b)(1) — ratable: market_discount × (days held /
            // days from acquisition to maturity), capped at full
            // market_discount.
            if input.days_from_acquisition_to_maturity == 0 {
                market_discount
            } else {
                let held = input.days_held.min(input.days_from_acquisition_to_maturity) as i64;
                let total = input.days_from_acquisition_to_maturity as i64;
                let computed = market_discount.saturating_mul(held) / total;
                computed.min(market_discount)
            }
        }
        AccrualMethod::ConstantYield => {
            // § 1276(b)(2) — caller-supplied § 1272(a) computation,
            // clamped at full market discount.
            notes.push(
                "§ 1276(b)(2) constant-yield election — using § 1272(a) OID yield-to-maturity \
                 formula; election is irrevocable as to this bond."
                    .to_string(),
            );
            input
                .constant_yield_accrual_cents
                .max(0)
                .min(market_discount)
        }
    };

    // § 1278(b) current-inclusion election — exclude prior-year
    // accrual from ordinary recharacterization.
    let current_period_accrual_subject_to_recharacterization = if input.current_inclusion_election {
        notes.push(
            "§ 1278(b) current-inclusion election applies — prior-year accrual already taxed as \
             ordinary income is excluded from the § 1276 disposition recharacterization to \
             avoid double inclusion."
                .to_string(),
        );
        accrued
            .saturating_sub(input.prior_years_accrual_already_taxed_cents.max(0))
            .max(0)
    } else {
        accrued
    };

    // Realized amount under § 1276(a)(2) — non-sale dispositions
    // treated as realizing FMV. Caller supplies FMV in
    // realized_amount_cents for that disposition type.
    let realized = input.realized_amount_cents;

    // Total gain on disposition (for Sale + NonSale paths).
    let total_gain = match input.disposition_type {
        DispositionType::Sale | DispositionType::NonSaleDisposition => {
            realized.saturating_sub(input.purchase_price_cents).max(0)
        }
        DispositionType::PartialPrincipalPayment => realized.max(0),
    };

    if matches!(input.disposition_type, DispositionType::NonSaleDisposition) {
        notes.push(
            "§ 1276(a)(2) — non-sale disposition (gift, distribution, etc.); taxpayer treated \
             as realizing an amount equal to the fair market value of the bond at disposition."
                .to_string(),
        );
    }

    if matches!(
        input.disposition_type,
        DispositionType::PartialPrincipalPayment
    ) {
        notes.push(
            "§ 1276(a)(3) — partial principal payment treated as ordinary income up to \
             accrued market discount; basis is reduced for the principal-payment portion."
                .to_string(),
        );
    }

    // § 1276(a)(1) — ordinary income capped at MIN(total gain,
    // accrued market discount net of prior-year inclusions).
    let ordinary = total_gain.min(current_period_accrual_subject_to_recharacterization);
    let capital_gain = total_gain.saturating_sub(ordinary).max(0);
    let treated_as_interest = ordinary > 0;

    if treated_as_interest {
        notes.push(
            "§ 1276(a)(4) — amount treated as ordinary income under § 1276(a)(1) or (3) is \
             treated as INTEREST for purposes of this title (with carve-outs for §§ 103, \
             871(a), 881, 1441, 1442, 6049)."
                .to_string(),
        );
    }

    let citation = match input.disposition_type {
        DispositionType::Sale => {
            "26 U.S.C. § 1276(a)(1) (general rule — disposition gain ordinary up to accrued \
             market discount); § 1276(b)(1) (ratable accrual default) or § 1276(b)(2) \
             (constant-yield election); § 1276(a)(4) (treatment as interest); § 1278(a)(2)(A) \
             (market discount definition)"
        }
        DispositionType::PartialPrincipalPayment => {
            "26 U.S.C. § 1276(a)(3) (partial principal payment ordinary up to accrued market \
             discount); § 1276(b)(1) or § 1276(b)(2) (accrual computation); § 1276(a)(4) \
             (treatment as interest); § 1278(a)(2)(A) (market discount definition)"
        }
        DispositionType::NonSaleDisposition => {
            "26 U.S.C. § 1276(a)(1) (general rule — disposition gain ordinary up to accrued \
             market discount) and § 1276(a)(2) (non-sale dispositions treated as realizing \
             FMV); § 1276(b)(1) or § 1276(b)(2) (accrual computation); § 1276(a)(4) (treatment \
             as interest); § 1278(a)(2)(A) (market discount definition)"
        }
    };

    Section1276Result {
        market_discount_cents: market_discount,
        accrued_market_discount_cents: accrued,
        ordinary_income_cents: ordinary,
        capital_gain_cents: capital_gain,
        treated_as_interest,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        purchase: i64,
        face: i64,
        days_total: u32,
        days_held: u32,
        disposition: DispositionType,
        realized: i64,
    ) -> Section1276Input {
        Section1276Input {
            purchase_price_cents: purchase,
            stated_redemption_at_maturity_cents: face,
            days_from_acquisition_to_maturity: days_total,
            days_held,
            disposition_type: disposition,
            realized_amount_cents: realized,
            accrual_method: AccrualMethod::RatableAccrual,
            constant_yield_accrual_cents: 0,
            current_inclusion_election: false,
            prior_years_accrual_already_taxed_cents: 0,
        }
    }

    // ── § 1276(b)(1) ratable accrual math ────────────────────────

    #[test]
    fn ratable_accrual_half_holding_period_half_discount() {
        // $1000 face, $900 basis → $100 discount.
        // Held 365 of 730 days = 50%. Accrued = $50.
        let r = compute(&input(
            90_000,
            100_000,
            730,
            365,
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.market_discount_cents, 10_000);
        assert_eq!(r.accrued_market_discount_cents, 5_000);
        // Total gain = 100_000 − 90_000 = 10_000.
        // Ordinary cap = min(10_000, 5_000) = 5_000.
        assert_eq!(r.ordinary_income_cents, 5_000);
        assert_eq!(r.capital_gain_cents, 5_000);
        assert!(r.treated_as_interest);
        assert!(r.citation.contains("§ 1276(a)(1)"));
        assert!(r.citation.contains("§ 1278(a)(2)(A)"));
    }

    #[test]
    fn ratable_accrual_full_holding_period_full_discount() {
        // Held entire 730 days to maturity → 100% accrued.
        let r = compute(&input(
            90_000,
            100_000,
            730,
            730,
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 10_000);
        assert_eq!(r.ordinary_income_cents, 10_000);
        assert_eq!(r.capital_gain_cents, 0);
    }

    #[test]
    fn ratable_accrual_held_longer_than_total_caps_at_full_discount() {
        // Days held > total days — should cap at full discount.
        let r = compute(&input(
            90_000,
            100_000,
            365,
            500, // beyond maturity
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 10_000);
    }

    #[test]
    fn ratable_accrual_zero_days_held_zero_accrued() {
        let r = compute(&input(
            90_000,
            100_000,
            365,
            0,
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 0);
        assert_eq!(r.ordinary_income_cents, 0);
    }

    #[test]
    fn ratable_accrual_zero_total_days_uses_full_discount() {
        // Edge case — maturity passed, denominator zero; degenerate
        // accrual takes the full discount.
        let r = compute(&input(
            90_000,
            100_000,
            0,
            0,
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 10_000);
    }

    // ── § 1276(a)(1) gain less than accrued ──────────────────────

    #[test]
    fn gain_less_than_accrued_ordinary_caps_at_gain() {
        // $100 market discount; held full period → accrued $100.
        // Sale at $950 → only $50 total gain. Ordinary = min(50, 100)
        // = 50; capital = 0.
        let r = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::Sale,
            95_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 10_000);
        assert_eq!(r.ordinary_income_cents, 5_000);
        assert_eq!(r.capital_gain_cents, 0);
    }

    #[test]
    fn loss_on_sale_no_ordinary_recharacterization() {
        // Sale at $850 → loss of $50; total gain is 0 (clamped).
        // Ordinary = 0; capital = 0 (the loss itself is computed
        // elsewhere via Schedule D).
        let r = compute(&input(
            90_000,
            100_000,
            365,
            200,
            DispositionType::Sale,
            85_000,
        ));
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_cents, 0);
        assert!(!r.treated_as_interest);
    }

    // ── No-discount path ────────────────────────────────────────

    #[test]
    fn purchase_at_or_above_face_no_market_discount() {
        // Premium bond — no § 1276 application.
        let r = compute(&input(
            100_000,
            95_000,
            365,
            200,
            DispositionType::Sale,
            100_000,
        ));
        assert_eq!(r.market_discount_cents, 0);
        assert_eq!(r.accrued_market_discount_cents, 0);
        assert_eq!(r.ordinary_income_cents, 0);
        // Total gain = 100_000 − 100_000 = 0.
        assert_eq!(r.capital_gain_cents, 0);
        assert!(r.citation.contains("§ 1278(a)(2)(A)"));
        assert!(r.citation.contains("inapplicable"));
    }

    #[test]
    fn purchase_at_face_no_discount() {
        let r = compute(&input(
            100_000,
            100_000,
            365,
            365,
            DispositionType::Sale,
            105_000,
        ));
        assert_eq!(r.market_discount_cents, 0);
        assert_eq!(r.ordinary_income_cents, 0);
        // Total gain = 105_000 − 100_000 = 5_000; all capital.
        assert_eq!(r.capital_gain_cents, 5_000);
    }

    // ── § 1276(a)(2) non-sale disposition ───────────────────────

    #[test]
    fn non_sale_disposition_uses_fmv_as_realized() {
        // Gift at FMV $97_000; basis $90_000 → realized gain $7_000.
        // Held full → accrued $10_000. Ordinary = min(7_000, 10_000)
        // = 7_000.
        let r = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::NonSaleDisposition,
            97_000,
        ));
        assert_eq!(r.ordinary_income_cents, 7_000);
        assert_eq!(r.capital_gain_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1276(a)(2)") && n.contains("fair market value")));
        assert!(r.citation.contains("§ 1276(a)(2)"));
    }

    // ── § 1276(a)(3) partial principal payment ──────────────────

    #[test]
    fn partial_principal_payment_ordinary_up_to_accrued() {
        // Partial principal $3_000; accrued $5_000 (half holding) →
        // ordinary $3_000; capital $0.
        let r = compute(&input(
            90_000,
            100_000,
            730,
            365,
            DispositionType::PartialPrincipalPayment,
            3_000,
        ));
        assert_eq!(r.accrued_market_discount_cents, 5_000);
        assert_eq!(r.ordinary_income_cents, 3_000);
        assert_eq!(r.capital_gain_cents, 0);
        assert!(r.citation.contains("§ 1276(a)(3)"));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1276(a)(3)") && n.contains("partial principal")));
    }

    #[test]
    fn partial_principal_payment_exceeding_accrued_caps() {
        // Partial principal $8_000; accrued $5_000 → ordinary $5_000;
        // capital $3_000.
        let r = compute(&input(
            90_000,
            100_000,
            730,
            365,
            DispositionType::PartialPrincipalPayment,
            8_000,
        ));
        assert_eq!(r.ordinary_income_cents, 5_000);
        assert_eq!(r.capital_gain_cents, 3_000);
    }

    // ── § 1276(b)(2) constant-yield election ────────────────────

    #[test]
    fn constant_yield_election_uses_caller_supplied_accrual() {
        let mut i = input(90_000, 100_000, 730, 365, DispositionType::Sale, 100_000);
        i.accrual_method = AccrualMethod::ConstantYield;
        // Constant-yield computation yields more than ratable would
        // (e.g., 6_000 vs ratable 5_000) for a back-loaded yield.
        i.constant_yield_accrual_cents = 6_000;
        let r = compute(&i);
        assert_eq!(r.accrued_market_discount_cents, 6_000);
        assert_eq!(r.ordinary_income_cents, 6_000);
        assert_eq!(r.capital_gain_cents, 4_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1276(b)(2)") && n.contains("§ 1272(a)")));
    }

    #[test]
    fn constant_yield_caller_supplied_exceeds_market_discount_caps() {
        // Caller-supplied accrual $15_000 must cap at $10_000 (full
        // market discount).
        let mut i = input(90_000, 100_000, 730, 365, DispositionType::Sale, 100_000);
        i.accrual_method = AccrualMethod::ConstantYield;
        i.constant_yield_accrual_cents = 15_000;
        let r = compute(&i);
        assert_eq!(r.accrued_market_discount_cents, 10_000);
    }

    #[test]
    fn constant_yield_negative_caller_supplied_clamps_at_zero() {
        let mut i = input(90_000, 100_000, 730, 365, DispositionType::Sale, 100_000);
        i.accrual_method = AccrualMethod::ConstantYield;
        i.constant_yield_accrual_cents = -500;
        let r = compute(&i);
        assert_eq!(r.accrued_market_discount_cents, 0);
        assert_eq!(r.ordinary_income_cents, 0);
    }

    // ── § 1278(b) current-inclusion election ────────────────────

    #[test]
    fn current_inclusion_election_subtracts_prior_year_accrual() {
        // Total accrued $10_000; prior-year taxed $4_000. Net
        // recharacterizable = $6_000. Total gain $10_000. Ordinary
        // = $6_000; capital = $4_000 (not $0).
        let mut i = input(90_000, 100_000, 365, 365, DispositionType::Sale, 100_000);
        i.current_inclusion_election = true;
        i.prior_years_accrual_already_taxed_cents = 4_000;
        let r = compute(&i);
        assert_eq!(r.accrued_market_discount_cents, 10_000);
        assert_eq!(r.ordinary_income_cents, 6_000);
        assert_eq!(r.capital_gain_cents, 4_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1278(b)") && n.contains("current-inclusion")));
    }

    #[test]
    fn current_inclusion_prior_year_equals_accrued_zero_recharacterization() {
        let mut i = input(90_000, 100_000, 365, 365, DispositionType::Sale, 100_000);
        i.current_inclusion_election = true;
        i.prior_years_accrual_already_taxed_cents = 10_000;
        let r = compute(&i);
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_cents, 10_000);
    }

    #[test]
    fn current_inclusion_prior_year_exceeds_accrued_does_not_negate() {
        let mut i = input(90_000, 100_000, 365, 365, DispositionType::Sale, 100_000);
        i.current_inclusion_election = true;
        i.prior_years_accrual_already_taxed_cents = 25_000;
        let r = compute(&i);
        // Saturating subtraction prevents negative; all $10_000 gain
        // is capital.
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_cents, 10_000);
    }

    // ── § 1276(a)(4) treated as interest flag ───────────────────

    #[test]
    fn treated_as_interest_only_when_ordinary_recharacterization_nonzero() {
        // No discount → no recharacterization → not treated as
        // interest.
        let no_discount = compute(&input(
            100_000,
            95_000,
            365,
            365,
            DispositionType::Sale,
            105_000,
        ));
        assert!(!no_discount.treated_as_interest);

        // Discount + held + gain → recharacterization → treated as
        // interest.
        let with_discount = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::Sale,
            100_000,
        ));
        assert!(with_discount.treated_as_interest);
    }

    // ── Multi-path regression invariants ────────────────────────

    #[test]
    fn ordinary_plus_capital_equals_total_gain_invariant() {
        // Sale + NonSale paths.
        for &disp in &[DispositionType::Sale, DispositionType::NonSaleDisposition] {
            let r = compute(&input(90_000, 100_000, 365, 200, disp, 100_000));
            let total_gain = 100_000_i64.saturating_sub(90_000).max(0);
            assert_eq!(
                r.ordinary_income_cents + r.capital_gain_cents,
                total_gain,
                "{:?}: ordinary + capital must equal total gain",
                disp,
            );
        }
    }

    #[test]
    fn ordinary_never_exceeds_accrued_market_discount_invariant() {
        for &disp in &[
            DispositionType::Sale,
            DispositionType::PartialPrincipalPayment,
            DispositionType::NonSaleDisposition,
        ] {
            for realized in [50_000_i64, 95_000, 99_000, 100_000, 110_000, 150_000] {
                let r = compute(&input(90_000, 100_000, 365, 365, disp, realized));
                assert!(
                    r.ordinary_income_cents <= r.accrued_market_discount_cents,
                    "{:?} realized={}: ordinary > accrued",
                    disp,
                    realized,
                );
            }
        }
    }

    #[test]
    fn citation_pins_subsection_per_disposition_type() {
        let sale = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::Sale,
            100_000,
        ));
        let partial = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::PartialPrincipalPayment,
            5_000,
        ));
        let non_sale = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::NonSaleDisposition,
            100_000,
        ));

        assert!(sale.citation.contains("§ 1276(a)(1)"));
        assert!(partial.citation.contains("§ 1276(a)(3)"));
        assert!(non_sale.citation.contains("§ 1276(a)(2)"));
        for r in [&sale, &partial, &non_sale] {
            assert!(r.citation.contains("§ 1276(a)(4)"));
            assert!(r.citation.contains("§ 1278(a)(2)(A)"));
        }
    }

    #[test]
    fn treated_as_interest_note_present_when_recharacterization_occurs() {
        let r = compute(&input(
            90_000,
            100_000,
            365,
            365,
            DispositionType::Sale,
            100_000,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1276(a)(4)") && n.contains("INTEREST")),
            "§ 1276(a)(4) interest-treatment note must appear when recharacterization occurs"
        );
    }
}
