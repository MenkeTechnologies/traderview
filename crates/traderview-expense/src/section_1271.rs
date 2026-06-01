//! IRC § 1271 — Retirement of debt instruments treated as paid in
//! exchange therefor (i.e., as a sale or exchange).
//!
//! The character-conversion rule for the END of a bond's life. When
//! a bond matures and the issuer pays the holder the redemption
//! amount, § 1271(a)(1) treats the payment as if the bond were
//! SOLD — so the gain or loss takes capital character (subject to
//! the carve-outs in § 1271(a)(2)–(a)(4) that recharacterize some
//! gain as ordinary).
//!
//! Companion to:
//!   - `section_1276` (market discount on bonds — ordinary
//!     recharacterization on disposition).
//!   - `section_1277` (interest-deduction deferral on carrying
//!     debt for market-discount bonds).
//!   - `section_1278` (definitions + § 1278(b) current-inclusion
//!     election).
//!
//! Five operative paths under § 1271:
//!
//!   § 1271(a)(1) — GENERAL RULE: Amounts received by the holder
//!     on retirement of any debt instrument are considered amounts
//!     received in exchange therefor. Default character is capital
//!     (subject to other Code provisions like § 1276 market
//!     discount recharacterization).
//!
//!   § 1271(a)(2) — INTENT-TO-CALL OID RECHARACTERIZATION: If the
//!     debt instrument was originally issued with intent to call
//!     before maturity, gain on sale or exchange (up to the OID
//!     amount not previously included in gross income) is treated
//!     as ORDINARY income. Carve-outs: does NOT apply to
//!     (i) tax-exempt obligations or (ii) holders who purchased
//!     the instrument at a premium.
//!
//!   § 1271(a)(3) — SHORT-TERM GOVERNMENT OBLIGATIONS: Fixed
//!     maturity ≤ 1 year from issue. Gain up to ratable share of
//!     acquisition discount → ordinary. Constant-yield election
//!     available.
//!
//!   § 1271(a)(4) — SHORT-TERM NONGOVERNMENT OBLIGATIONS: Fixed
//!     maturity ≤ 1 year. Gain up to ratable share of OID →
//!     ordinary. Constant-yield election available.
//!
//!   § 1271(b) — NATURAL-PERSON ISSUER EXCEPTION: § 1271 does not
//!     apply to obligations issued by a natural person before
//!     June 9, 1997 (with carve-out for obligations purchased
//!     after that date).
//!
//!   § 1271(c) — NO DOUBLE INCLUSION: § 1271, § 1272, and § 1286
//!     do not require inclusion of any amount that was previously
//!     includible in gross income.
//!
//! Citations: 26 U.S.C. § 1271(a)(1) (general sale-or-exchange
//! treatment); § 1271(a)(2) (intent-to-call OID ordinary
//! recharacterization with tax-exempt + premium carve-outs);
//! § 1271(a)(3) (short-term government obligations); § 1271(a)(4)
//! (short-term nongovernment obligations); § 1271(b) (natural-
//! person issuer pre-June-9-1997 exception); § 1271(c) (no double
//! inclusion); § 1272 (OID current inclusion); § 1276 + § 1277 +
//! § 1278 (market-discount-bond trilogy).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DebtInstrumentType {
    /// Standard debt instrument — § 1271(a)(1) default capital
    /// character on retirement.
    Standard,
    /// Debt instrument originally issued with intent to call
    /// before maturity — § 1271(a)(2) ordinary recharacterization
    /// up to OID.
    IntentToCallOID,
    /// Short-term government obligation (maturity ≤ 1 year) —
    /// § 1271(a)(3) ratable acquisition-discount recharacterization.
    ShortTermGovernment,
    /// Short-term nongovernment obligation (maturity ≤ 1 year) —
    /// § 1271(a)(4) ratable OID recharacterization.
    ShortTermNonGovernment,
    /// Obligation issued by a natural person before June 9, 1997 —
    /// § 1271(b) exception. § 1271 does not apply.
    NaturalPersonPre1997,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1271Input {
    pub debt_type: DebtInstrumentType,
    /// Taxpayer's purchase price (basis) at acquisition (cents).
    pub purchase_price_cents: i64,
    /// Amount received on retirement at maturity (cents).
    pub redemption_amount_cents: i64,
    /// Original issue discount (cents) — used by § 1271(a)(2)
    /// intent-to-call recharacterization cap.
    pub original_issue_discount_cents: i64,
    /// OID amount already included in gross income in prior years
    /// (cents). Subtracted from the § 1271(a)(2) recharacterization
    /// cap per § 1271(c) no-double-inclusion.
    pub oid_previously_included_cents: i64,
    /// Acquisition discount for short-term government obligations
    /// (cents) — § 1271(a)(3).
    pub acquisition_discount_cents: i64,
    /// Ratable accrued discount for the holding period of a
    /// short-term obligation (cents) — § 1271(a)(3) + (a)(4) cap.
    pub ratable_short_term_accrual_cents: i64,
    /// Whether the taxpayer purchased the instrument at a PREMIUM
    /// (above face) — § 1271(a)(2) carve-out from intent-to-call
    /// ordinary recharacterization.
    pub purchased_at_premium: bool,
    /// Whether the obligation is tax-exempt — § 1271(a)(2)
    /// carve-out from intent-to-call ordinary recharacterization.
    pub tax_exempt: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1271Result {
    /// Total gain or loss on retirement (cents) — redemption minus
    /// basis.
    pub gain_or_loss_cents: i64,
    /// Portion of gain recharacterized as ordinary income under
    /// § 1271(a)(2)–(a)(4) (cents). Zero where no recharacterization
    /// applies (Standard path, premium buyer, tax-exempt, or
    /// § 1271(b) exception).
    pub ordinary_income_cents: i64,
    /// Residual capital gain or loss after ordinary
    /// recharacterization (cents).
    pub capital_gain_or_loss_cents: i64,
    /// True where § 1271(a)(1) treats the retirement as a sale or
    /// exchange. False where § 1271(b) natural-person pre-1997
    /// exception bars the section.
    pub treated_as_sale_or_exchange: bool,
    /// True where the § 1271(b) natural-person pre-June-9-1997
    /// issuer exception applies.
    pub section_1271b_exception_applies: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1271Input) -> Section1271Result {
    let mut notes: Vec<String> = Vec::new();

    let gain_or_loss = input
        .redemption_amount_cents
        .saturating_sub(input.purchase_price_cents);

    // § 1271(b) — natural-person pre-June-9-1997 exception.
    if matches!(input.debt_type, DebtInstrumentType::NaturalPersonPre1997) {
        notes.push(
            "§ 1271(b) — obligation issued by a natural person before June 9, 1997; § 1271 \
             does not apply. Holder's retirement payment is treated under the pre-§ 1271 \
             rules (ordinary character of the principal-repayment portion)."
                .to_string(),
        );
        return Section1271Result {
            gain_or_loss_cents: gain_or_loss,
            ordinary_income_cents: 0,
            capital_gain_or_loss_cents: gain_or_loss,
            treated_as_sale_or_exchange: false,
            section_1271b_exception_applies: true,
            citation: "26 U.S.C. § 1271(b) (natural-person issuer pre-June-9-1997 exception)",
            notes,
        };
    }

    // § 1271(a)(2) intent-to-call OID recharacterization.
    if matches!(input.debt_type, DebtInstrumentType::IntentToCallOID) {
        // Carve-outs: tax-exempt or premium-purchaser → no ordinary
        // recharacterization.
        if input.tax_exempt || input.purchased_at_premium {
            if input.tax_exempt {
                notes.push(
                    "§ 1271(a)(2) tax-exempt carve-out — § 1271(a)(2) ordinary recharacterization \
                     does not apply to tax-exempt obligations."
                        .to_string(),
                );
            }
            if input.purchased_at_premium {
                notes.push(
                    "§ 1271(a)(2) premium-buyer carve-out — § 1271(a)(2) ordinary \
                     recharacterization does not apply where the holder purchased the \
                     instrument at a premium."
                        .to_string(),
                );
            }
            return Section1271Result {
                gain_or_loss_cents: gain_or_loss,
                ordinary_income_cents: 0,
                capital_gain_or_loss_cents: gain_or_loss,
                treated_as_sale_or_exchange: true,
                section_1271b_exception_applies: false,
                citation: "26 U.S.C. § 1271(a)(1) (general sale-or-exchange treatment); \
                           § 1271(a)(2) carve-out (tax-exempt or premium-buyer — ordinary \
                           recharacterization does not apply)",
                notes,
            };
        }
        // § 1271(c) — recharacterization cap reduced by OID
        // previously included in gross income to prevent double
        // inclusion.
        let remaining_oid_cap = input
            .original_issue_discount_cents
            .saturating_sub(input.oid_previously_included_cents.max(0))
            .max(0);
        let ordinary = gain_or_loss.max(0).min(remaining_oid_cap);
        let capital = gain_or_loss.saturating_sub(ordinary);
        notes.push(
            "§ 1271(a)(2) — intent-to-call OID instrument; gain up to remaining OID (net of \
             prior-year inclusions under § 1271(c)) recharacterized as ordinary income."
                .to_string(),
        );
        return Section1271Result {
            gain_or_loss_cents: gain_or_loss,
            ordinary_income_cents: ordinary,
            capital_gain_or_loss_cents: capital,
            treated_as_sale_or_exchange: true,
            section_1271b_exception_applies: false,
            citation: "26 U.S.C. § 1271(a)(1) (general sale-or-exchange treatment); \
                       § 1271(a)(2) (intent-to-call OID ordinary recharacterization up to OID \
                       net of § 1271(c) prior inclusion); § 1271(c) (no double inclusion)",
            notes,
        };
    }

    // § 1271(a)(3) + (a)(4) short-term obligations — ratable
    // acquisition-discount / OID recharacterization.
    if matches!(
        input.debt_type,
        DebtInstrumentType::ShortTermGovernment | DebtInstrumentType::ShortTermNonGovernment
    ) {
        let cap = match input.debt_type {
            DebtInstrumentType::ShortTermGovernment => input.acquisition_discount_cents,
            DebtInstrumentType::ShortTermNonGovernment => input.original_issue_discount_cents,
            _ => 0,
        }
        .max(0);
        let ratable = input.ratable_short_term_accrual_cents.max(0).min(cap);
        let ordinary = gain_or_loss.max(0).min(ratable);
        let capital = gain_or_loss.saturating_sub(ordinary);
        let citation = match input.debt_type {
            DebtInstrumentType::ShortTermGovernment => {
                notes.push(
                    "§ 1271(a)(3) — short-term government obligation (≤ 1 year to maturity); \
                     gain up to ratable share of acquisition discount recharacterized as \
                     ordinary income."
                        .to_string(),
                );
                "26 U.S.C. § 1271(a)(1) (general sale-or-exchange treatment); § 1271(a)(3) \
                 (short-term government obligation — ordinary up to ratable acquisition \
                 discount); § 1271(c) (no double inclusion)"
            }
            DebtInstrumentType::ShortTermNonGovernment => {
                notes.push(
                    "§ 1271(a)(4) — short-term nongovernment obligation (≤ 1 year to maturity); \
                     gain up to ratable share of OID recharacterized as ordinary income."
                        .to_string(),
                );
                "26 U.S.C. § 1271(a)(1) (general sale-or-exchange treatment); § 1271(a)(4) \
                 (short-term nongovernment obligation — ordinary up to ratable OID); \
                 § 1271(c) (no double inclusion)"
            }
            _ => unreachable!(),
        };
        return Section1271Result {
            gain_or_loss_cents: gain_or_loss,
            ordinary_income_cents: ordinary,
            capital_gain_or_loss_cents: capital,
            treated_as_sale_or_exchange: true,
            section_1271b_exception_applies: false,
            citation,
            notes,
        };
    }

    // § 1271(a)(1) Standard path — capital character on retirement.
    notes.push(
        "§ 1271(a)(1) — retirement of standard debt instrument treated as sale or exchange; \
         gain or loss is capital (subject to other Code provisions such as § 1276 market-\
         discount ordinary recharacterization)."
            .to_string(),
    );
    Section1271Result {
        gain_or_loss_cents: gain_or_loss,
        ordinary_income_cents: 0,
        capital_gain_or_loss_cents: gain_or_loss,
        treated_as_sale_or_exchange: true,
        section_1271b_exception_applies: false,
        citation: "26 U.S.C. § 1271(a)(1) (general rule — retirement of debt instrument \
                   treated as sale or exchange; capital character subject to other Code \
                   provisions)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        debt_type: DebtInstrumentType,
        purchase: i64,
        redemption: i64,
        oid: i64,
        oid_prior: i64,
        acq_disc: i64,
        ratable: i64,
        premium: bool,
        tax_exempt: bool,
    ) -> Section1271Input {
        Section1271Input {
            debt_type,
            purchase_price_cents: purchase,
            redemption_amount_cents: redemption,
            original_issue_discount_cents: oid,
            oid_previously_included_cents: oid_prior,
            acquisition_discount_cents: acq_disc,
            ratable_short_term_accrual_cents: ratable,
            purchased_at_premium: premium,
            tax_exempt: tax_exempt,
        }
    }

    // ── § 1271(a)(1) standard rule ──────────────────────────────

    #[test]
    fn standard_retirement_treated_as_sale_or_exchange_capital() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            90_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        assert!(r.treated_as_sale_or_exchange);
        assert!(!r.section_1271b_exception_applies);
        assert_eq!(r.gain_or_loss_cents, 10_000);
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, 10_000);
        assert!(r.citation.contains("§ 1271(a)(1)"));
    }

    #[test]
    fn standard_retirement_at_basis_zero_gain() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            100_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gain_or_loss_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, 0);
    }

    #[test]
    fn standard_retirement_loss_capital() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            110_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.gain_or_loss_cents, -10_000);
        assert_eq!(r.capital_gain_or_loss_cents, -10_000);
        assert_eq!(r.ordinary_income_cents, 0);
    }

    // ── § 1271(a)(2) intent-to-call OID ─────────────────────────

    #[test]
    fn intent_to_call_oid_full_ordinary_up_to_oid_amount() {
        // Basis 90_000, redemption 100_000 → gain 10_000.
        // OID 8_000, prior 0 → cap 8_000.
        // Ordinary = min(10_000, 8_000) = 8_000; capital = 2_000.
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 8_000);
        assert_eq!(r.capital_gain_or_loss_cents, 2_000);
        assert!(r.citation.contains("§ 1271(a)(2)"));
        assert!(r.citation.contains("§ 1271(c)"));
    }

    #[test]
    fn intent_to_call_gain_less_than_oid_cap_full_ordinary() {
        // Basis 95_000, redemption 100_000 → gain 5_000.
        // OID 8_000 → cap 8_000.
        // Ordinary = min(5_000, 8_000) = 5_000; capital = 0.
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            95_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 5_000);
        assert_eq!(r.capital_gain_or_loss_cents, 0);
    }

    #[test]
    fn intent_to_call_prior_year_oid_inclusion_reduces_cap_per_1271c() {
        // OID 8_000; prior years already included 5_000.
        // Remaining cap = 3_000.
        // Gain 10_000 → ordinary 3_000; capital 7_000.
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            5_000,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 3_000);
        assert_eq!(r.capital_gain_or_loss_cents, 7_000);
    }

    #[test]
    fn intent_to_call_prior_year_oid_exceeds_oid_zero_recharacterization() {
        // Prior 10_000, OID 8_000 → cap saturates at 0.
        // Gain 10_000 → ordinary 0; capital 10_000.
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            10_000,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, 10_000);
    }

    #[test]
    fn intent_to_call_tax_exempt_carve_out_no_recharacterization() {
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            true,
        ));
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, 10_000);
        assert!(r.citation.contains("§ 1271(a)(2) carve-out"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1271(a)(2) tax-exempt carve-out"))
        );
    }

    #[test]
    fn intent_to_call_premium_buyer_carve_out_no_recharacterization() {
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            true,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, 10_000);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1271(a)(2) premium-buyer carve-out"))
        );
    }

    #[test]
    fn intent_to_call_loss_no_ordinary_recharacterization() {
        // Loss → ordinary = max(gain, 0) = 0; capital = loss.
        let r = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            110_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 0);
        assert_eq!(r.capital_gain_or_loss_cents, -10_000);
    }

    // ── § 1271(a)(3) short-term government obligations ─────────

    #[test]
    fn short_term_government_obligation_ratable_acquisition_discount_ordinary() {
        // Basis 95_000, redemption 100_000 → gain 5_000.
        // Acq disc 5_000; ratable accrual 2_000.
        // Ordinary = min(5_000, min(2_000, 5_000)) = 2_000;
        // capital = 3_000.
        let r = compute(&input(
            DebtInstrumentType::ShortTermGovernment,
            95_000,
            100_000,
            0,
            0,
            5_000,
            2_000,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 2_000);
        assert_eq!(r.capital_gain_or_loss_cents, 3_000);
        assert!(r.citation.contains("§ 1271(a)(3)"));
    }

    #[test]
    fn short_term_government_ratable_exceeds_acquisition_discount_caps_at_acq() {
        // Ratable 10_000 > Acq disc 5_000 → cap = 5_000.
        // Gain 5_000 → ordinary 5_000; capital 0.
        let r = compute(&input(
            DebtInstrumentType::ShortTermGovernment,
            95_000,
            100_000,
            0,
            0,
            5_000,
            10_000,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 5_000);
        assert_eq!(r.capital_gain_or_loss_cents, 0);
    }

    // ── § 1271(a)(4) short-term nongovernment obligations ──────

    #[test]
    fn short_term_nongovernment_ratable_oid_ordinary() {
        // Basis 95_000, redemption 100_000 → gain 5_000.
        // OID 5_000; ratable 2_500.
        // Ordinary = 2_500; capital = 2_500.
        let r = compute(&input(
            DebtInstrumentType::ShortTermNonGovernment,
            95_000,
            100_000,
            5_000,
            0,
            0,
            2_500,
            false,
            false,
        ));
        assert_eq!(r.ordinary_income_cents, 2_500);
        assert_eq!(r.capital_gain_or_loss_cents, 2_500);
        assert!(r.citation.contains("§ 1271(a)(4)"));
    }

    // ── § 1271(b) natural-person pre-June-9-1997 exception ─────

    #[test]
    fn natural_person_pre_1997_section_1271_does_not_apply() {
        let r = compute(&input(
            DebtInstrumentType::NaturalPersonPre1997,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        assert!(!r.treated_as_sale_or_exchange);
        assert!(r.section_1271b_exception_applies);
        assert_eq!(r.ordinary_income_cents, 0);
        assert!(r.citation.contains("§ 1271(b)"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1271(b)") && n.contains("June 9, 1997"))
        );
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn ordinary_plus_capital_equals_gain_invariant() {
        // Across all paths and gain levels, ordinary + capital must
        // sum exactly to gain_or_loss.
        for debt_type in [
            DebtInstrumentType::Standard,
            DebtInstrumentType::IntentToCallOID,
            DebtInstrumentType::ShortTermGovernment,
            DebtInstrumentType::ShortTermNonGovernment,
        ] {
            for (purchase, redemption) in [
                (90_000_i64, 100_000),
                (95_000, 100_000),
                (100_000, 100_000),
                (110_000, 100_000),
            ] {
                let r = compute(&input(
                    debt_type,
                    purchase,
                    redemption,
                    8_000,
                    0,
                    5_000,
                    2_000,
                    false,
                    false,
                ));
                assert_eq!(
                    r.ordinary_income_cents + r.capital_gain_or_loss_cents,
                    r.gain_or_loss_cents,
                    "{:?} purchase={purchase} redemption={redemption}: \
                     ordinary + capital must equal gain",
                    debt_type,
                );
            }
        }
    }

    #[test]
    fn ordinary_never_exceeds_gain_invariant() {
        // Ordinary recharacterization cannot exceed total gain
        // (no negative capital except where loss path).
        for debt_type in [
            DebtInstrumentType::Standard,
            DebtInstrumentType::IntentToCallOID,
            DebtInstrumentType::ShortTermGovernment,
            DebtInstrumentType::ShortTermNonGovernment,
        ] {
            let r = compute(&input(
                debt_type,
                90_000,
                100_000,
                50_000,
                0,
                50_000,
                50_000,
                false,
                false,
            ));
            assert!(
                r.ordinary_income_cents <= r.gain_or_loss_cents,
                "{:?}: ordinary > gain",
                debt_type,
            );
        }
    }

    #[test]
    fn only_natural_person_pre_1997_bypasses_sale_or_exchange_invariant() {
        for debt_type in [
            DebtInstrumentType::Standard,
            DebtInstrumentType::IntentToCallOID,
            DebtInstrumentType::ShortTermGovernment,
            DebtInstrumentType::ShortTermNonGovernment,
        ] {
            let r = compute(&input(
                debt_type,
                90_000,
                100_000,
                8_000,
                0,
                5_000,
                2_000,
                false,
                false,
            ));
            assert!(
                r.treated_as_sale_or_exchange,
                "{:?}: must be treated as sale/exchange",
                debt_type,
            );
            assert!(!r.section_1271b_exception_applies);
        }
        let r = compute(&input(
            DebtInstrumentType::NaturalPersonPre1997,
            90_000,
            100_000,
            8_000,
            0,
            5_000,
            2_000,
            false,
            false,
        ));
        assert!(!r.treated_as_sale_or_exchange);
        assert!(r.section_1271b_exception_applies);
    }

    #[test]
    fn citation_pins_subsection_per_debt_type() {
        let standard = compute(&input(
            DebtInstrumentType::Standard,
            90_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        let intent_to_call = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        let stg = compute(&input(
            DebtInstrumentType::ShortTermGovernment,
            95_000,
            100_000,
            0,
            0,
            5_000,
            2_000,
            false,
            false,
        ));
        let stng = compute(&input(
            DebtInstrumentType::ShortTermNonGovernment,
            95_000,
            100_000,
            5_000,
            0,
            0,
            2_500,
            false,
            false,
        ));
        let np = compute(&input(
            DebtInstrumentType::NaturalPersonPre1997,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));

        assert!(standard.citation.contains("§ 1271(a)(1)"));
        assert!(intent_to_call.citation.contains("§ 1271(a)(2)"));
        assert!(stg.citation.contains("§ 1271(a)(3)"));
        assert!(stng.citation.contains("§ 1271(a)(4)"));
        assert!(np.citation.contains("§ 1271(b)"));
    }

    #[test]
    fn carve_outs_only_apply_to_intent_to_call_path_invariant() {
        // For Standard path, premium and tax_exempt are no-ops.
        let std_with = compute(&input(
            DebtInstrumentType::Standard,
            90_000,
            100_000,
            0,
            0,
            0,
            0,
            true,
            true,
        ));
        let std_without = compute(&input(
            DebtInstrumentType::Standard,
            90_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(std_with, std_without);

        // For IntentToCallOID, premium or tax_exempt zeros ordinary.
        let itc_premium = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            true,
            false,
        ));
        let itc_no_premium = compute(&input(
            DebtInstrumentType::IntentToCallOID,
            90_000,
            100_000,
            8_000,
            0,
            0,
            0,
            false,
            false,
        ));
        assert_eq!(itc_premium.ordinary_income_cents, 0);
        assert_eq!(itc_no_premium.ordinary_income_cents, 8_000);
    }

    #[test]
    fn cross_reference_note_to_section_1276_in_standard_path() {
        let r = compute(&input(
            DebtInstrumentType::Standard,
            90_000,
            100_000,
            0,
            0,
            0,
            0,
            false,
            false,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1276") && n.contains("market-discount")),
            "Standard-path note must cross-reference § 1276"
        );
    }
}
