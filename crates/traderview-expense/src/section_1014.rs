//! IRC §1014 — Basis of property acquired from a decedent.
//!
//! The single most powerful rule in the Internal Revenue Code for buy-
//! and-hold investors and the foundation of every "die with low basis"
//! estate-planning strategy. When property passes from a decedent to an
//! heir, the heir's basis is **stepped up (or down) to the fair market
//! value on the date of death** under §1014(a)(1). All gain or loss that
//! accrued during the decedent's lifetime is permanently eliminated.
//!
//! Four exceptions / refinements matter:
//!
//! **§1014(c) — Income in respect of decedent (IRD).**  No step-up.
//! Applies to IRA distributions, accrued bond interest, deferred
//! compensation, US savings bond interest, etc. The decedent's adjusted
//! basis carries over; the heir still recognizes the embedded ordinary
//! income on receipt.
//!
//! **§1014(e) — One-year anti-abuse clawback.**  If the decedent
//! acquired the property by gift within 1 year of death AND the property
//! passes back to the donor (or the donor's spouse), there is no
//! step-up. Heir's basis = decedent's adjusted basis. Blocks the
//! "deathbed-gift-to-grandma" basis-laundering strategy.
//!
//! **§1014(f) — Consistent basis with estate tax return.**  Heir's
//! basis cannot exceed the FMV reported on Form 706 (the estate tax
//! return). Closes the "report low for estate tax, claim high for
//! income tax" arbitrage that historically existed. Capped by the
//! consistent-basis ceiling; floor (low Form 706) wins.
//!
//! **§2032 — Alternate valuation date election.**  The executor may
//! elect to value the estate as of 6 months after DOD instead of DOD
//! itself, BUT only if the election lowers BOTH (a) the gross estate
//! AND (b) the federal estate tax. The election applies to ALL property
//! in the estate, not item-by-item. When valid, the alternate-date FMV
//! becomes the heir's basis under §1014(a)(2).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1014Input {
    pub decedents_adjusted_basis: Decimal,
    pub fmv_at_dod: Decimal,
    pub date_of_death: NaiveDate,
    /// FMV 6 months after DOD. Required if `alternate_valuation_election_made`
    /// is `true`; ignored otherwise.
    pub fmv_at_alternate_valuation_date: Option<Decimal>,
    /// True if executor elected §2032 alternate valuation on Form 706.
    pub alternate_valuation_election_made: bool,
    /// True if the §2032 election actually lowered BOTH gross estate AND
    /// federal estate tax (the validity condition). Election with this
    /// false is invalid and falls back to DOD valuation.
    pub election_lowers_both_gross_estate_and_tax: bool,
    /// True if the decedent received this property as a gift within 1
    /// year of death. Required for §1014(e) check.
    pub decedent_received_as_gift_within_one_year: bool,
    /// True if the property passes back to the original donor or the
    /// donor's spouse. Combined with the 1-year window, triggers §1014(e).
    pub passes_back_to_donor_or_spouse: bool,
    /// True for income in respect of decedent (IRA distributions,
    /// deferred comp, accrued bond interest). §1014(c) denies step-up.
    pub is_income_in_respect_of_decedent: bool,
    /// FMV reported on Form 706 estate tax return, if filed. §1014(f)
    /// caps the heir's basis at this value when Some.
    pub fmv_reported_on_form_706: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section1014Rule {
    /// §1014(a)(1) — standard step-up to FMV at DOD.
    StandardStepUp,
    /// §1014(a)(2) — step-up to FMV at alternate valuation date (6 months
    /// after DOD), §2032 election.
    AlternateValuationDateStepUp,
    /// §1014(e) — 1-year anti-abuse clawback. No step-up; basis stays
    /// at decedent's adjusted basis.
    OneYearClawback,
    /// §1014(c) — income in respect of decedent. No step-up; basis stays
    /// at decedent's adjusted basis. Ordinary income on receipt.
    IncomeInRespectOfDecedent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValuationUsed {
    DateOfDeath,
    AlternateValuationDate,
    DecedentsAdjustedBasis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1014Result {
    pub heirs_basis: Decimal,
    /// Embedded gain (or loss when negative) eliminated by the step-up.
    /// `fmv_used - decedent's_basis` for step-up cases; zero for clawback
    /// and IRD because no step-up applies.
    pub unrealized_gain_eliminated: Decimal,
    pub rule_applied: Section1014Rule,
    pub valuation_used: ValuationUsed,
    /// True if the Form 706 consistent-basis cap under §1014(f) reduced
    /// the heir's basis below the raw FMV.
    pub form_706_consistent_basis_cap_applied: bool,
    /// True if a §2032 election was claimed but invalid (didn't lower
    /// both gross estate and tax) and the compute fell back to DOD FMV.
    pub alternate_valuation_election_invalid_fallback: bool,
    pub note: String,
}

pub fn compute(input: &Section1014Input) -> Section1014Result {
    // Order matters: §1014(c) IRD and §1014(e) clawback both deny the
    // step-up entirely. Check them BEFORE the valuation-date logic so a
    // §2032 election doesn't accidentally override the no-step-up rules.

    if input.is_income_in_respect_of_decedent {
        return Section1014Result {
            heirs_basis: input.decedents_adjusted_basis,
            unrealized_gain_eliminated: Decimal::ZERO,
            rule_applied: Section1014Rule::IncomeInRespectOfDecedent,
            valuation_used: ValuationUsed::DecedentsAdjustedBasis,
            form_706_consistent_basis_cap_applied: false,
            alternate_valuation_election_invalid_fallback: false,
            note: format!(
                "§1014(c) — income in respect of decedent: NO step-up. Heir's basis = decedent's ${} adjusted basis; ordinary income recognized on receipt",
                input.decedents_adjusted_basis.round_dp(2)
            ),
        };
    }

    if input.decedent_received_as_gift_within_one_year && input.passes_back_to_donor_or_spouse {
        return Section1014Result {
            heirs_basis: input.decedents_adjusted_basis,
            unrealized_gain_eliminated: Decimal::ZERO,
            rule_applied: Section1014Rule::OneYearClawback,
            valuation_used: ValuationUsed::DecedentsAdjustedBasis,
            form_706_consistent_basis_cap_applied: false,
            alternate_valuation_election_invalid_fallback: false,
            note: format!(
                "§1014(e) anti-abuse — decedent received property by gift within 1 year of death AND property passes back to donor/spouse: NO step-up. Heir's basis = decedent's ${} adjusted basis",
                input.decedents_adjusted_basis.round_dp(2)
            ),
        };
    }

    // Step-up path. Pick valuation: §2032 alternate vs DOD.
    let (fmv_pre_cap, valuation, rule, fallback_flag) = match (
        input.alternate_valuation_election_made,
        input.election_lowers_both_gross_estate_and_tax,
        input.fmv_at_alternate_valuation_date,
    ) {
        (true, true, Some(av_fmv)) => (
            av_fmv,
            ValuationUsed::AlternateValuationDate,
            Section1014Rule::AlternateValuationDateStepUp,
            false,
        ),
        (true, false, _) => (
            input.fmv_at_dod,
            ValuationUsed::DateOfDeath,
            Section1014Rule::StandardStepUp,
            true, // invalid election: didn't lower both
        ),
        (true, true, None) => (
            input.fmv_at_dod,
            ValuationUsed::DateOfDeath,
            Section1014Rule::StandardStepUp,
            true, // election claimed but no alternate FMV provided
        ),
        _ => (
            input.fmv_at_dod,
            ValuationUsed::DateOfDeath,
            Section1014Rule::StandardStepUp,
            false,
        ),
    };

    // §1014(f) consistent basis cap: heir's basis can't exceed Form 706.
    let (heirs_basis, cap_applied) = match input.fmv_reported_on_form_706 {
        Some(form_706_fmv) if form_706_fmv < fmv_pre_cap => (form_706_fmv, true),
        _ => (fmv_pre_cap, false),
    };

    let gain_eliminated = heirs_basis - input.decedents_adjusted_basis;

    let valuation_phrase = match valuation {
        ValuationUsed::DateOfDeath => format!("DOD ({})", input.date_of_death),
        ValuationUsed::AlternateValuationDate => {
            "§2032 alternate valuation date (6 months after DOD)".to_string()
        }
        ValuationUsed::DecedentsAdjustedBasis => "decedent's adjusted basis".to_string(),
    };

    let cap_phrase = if cap_applied {
        format!(
            " (capped by §1014(f) Form 706 FMV of ${})",
            heirs_basis.round_dp(2)
        )
    } else {
        String::new()
    };
    let fallback_phrase = if fallback_flag {
        " — §2032 election claimed but invalid; fell back to DOD valuation"
    } else {
        ""
    };

    Section1014Result {
        heirs_basis,
        unrealized_gain_eliminated: gain_eliminated,
        rule_applied: rule,
        valuation_used: valuation,
        form_706_consistent_basis_cap_applied: cap_applied,
        alternate_valuation_election_invalid_fallback: fallback_flag,
        note: format!(
            "§1014 step-up from valuation at {} = ${}{}; heir's basis = ${}; ${} unrealized gain/loss eliminated{}",
            valuation_phrase,
            fmv_pre_cap.round_dp(2),
            cap_phrase,
            heirs_basis.round_dp(2),
            gain_eliminated.round_dp(2),
            fallback_phrase,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section1014Input {
        Section1014Input {
            decedents_adjusted_basis: dec!(10_000),
            fmv_at_dod: dec!(100_000),
            date_of_death: d(2026, 3, 15),
            fmv_at_alternate_valuation_date: None,
            alternate_valuation_election_made: false,
            election_lowers_both_gross_estate_and_tax: false,
            decedent_received_as_gift_within_one_year: false,
            passes_back_to_donor_or_spouse: false,
            is_income_in_respect_of_decedent: false,
            fmv_reported_on_form_706: None,
        }
    }

    #[test]
    fn standard_step_up_to_dod_fmv() {
        // $10k basis, $100k FMV at DOD → heir's basis = $100k, $90k
        // unrealized gain permanently eliminated. The headline rule.
        let r = compute(&base());
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert_eq!(r.unrealized_gain_eliminated, dec!(90_000));
        assert_eq!(r.rule_applied, Section1014Rule::StandardStepUp);
        assert_eq!(r.valuation_used, ValuationUsed::DateOfDeath);
        assert!(!r.form_706_consistent_basis_cap_applied);
    }

    #[test]
    fn step_down_works_basis_drops_to_lower_fmv() {
        // $100k basis, $50k FMV → step-DOWN to $50k. Loss of $50k is
        // also eliminated — the rule cuts both ways.
        let mut i = base();
        i.decedents_adjusted_basis = dec!(100_000);
        i.fmv_at_dod = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(50_000));
        assert_eq!(r.unrealized_gain_eliminated, dec!(-50_000));
        assert_eq!(r.rule_applied, Section1014Rule::StandardStepUp);
    }

    #[test]
    fn ird_property_gets_no_step_up_per_subsection_c() {
        // §1014(c): IRA distribution / accrued interest / deferred comp.
        // Heir takes decedent's adjusted basis and recognizes the income
        // on receipt. No gain elimination.
        let mut i = base();
        i.is_income_in_respect_of_decedent = true;
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(10_000)); // unchanged
        assert_eq!(r.unrealized_gain_eliminated, Decimal::ZERO);
        assert_eq!(r.rule_applied, Section1014Rule::IncomeInRespectOfDecedent);
        assert!(r.note.contains("§1014(c)"));
    }

    #[test]
    fn one_year_clawback_denies_step_up() {
        // §1014(e): decedent received as gift within 1 year, property
        // returns to donor. Anti-abuse — no step-up.
        let mut i = base();
        i.decedent_received_as_gift_within_one_year = true;
        i.passes_back_to_donor_or_spouse = true;
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(10_000));
        assert_eq!(r.rule_applied, Section1014Rule::OneYearClawback);
        assert!(r.note.contains("§1014(e)"));
    }

    #[test]
    fn one_year_clawback_does_not_trigger_when_property_goes_to_other_heir() {
        // Both conditions must be true. If the property goes to someone
        // OTHER than the original donor (e.g., to a child of the
        // decedent), §1014(e) doesn't apply. Standard step-up applies.
        let mut i = base();
        i.decedent_received_as_gift_within_one_year = true;
        i.passes_back_to_donor_or_spouse = false;
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert_eq!(r.rule_applied, Section1014Rule::StandardStepUp);
    }

    #[test]
    fn one_year_clawback_does_not_trigger_outside_one_year_window() {
        // Both conditions must be true. If the gift was MORE than 1 year
        // before death, the clawback doesn't apply even if the property
        // returns to the donor.
        let mut i = base();
        i.decedent_received_as_gift_within_one_year = false;
        i.passes_back_to_donor_or_spouse = true;
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert_eq!(r.rule_applied, Section1014Rule::StandardStepUp);
    }

    #[test]
    fn alternate_valuation_election_valid_uses_av_fmv() {
        // §2032 — executor elects 6-month alternate; election lowered
        // both gross estate and federal tax. Heir's basis = alternate
        // FMV ($75k), not DOD FMV ($100k).
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(75_000));
        assert_eq!(r.rule_applied, Section1014Rule::AlternateValuationDateStepUp);
        assert_eq!(r.valuation_used, ValuationUsed::AlternateValuationDate);
        assert!(!r.alternate_valuation_election_invalid_fallback);
    }

    #[test]
    fn alternate_valuation_election_invalid_falls_back_to_dod() {
        // §2032 requires BOTH gross estate AND federal tax to decrease.
        // If only the gross estate decreased (because no federal tax was
        // owed under exemption), the election is invalid. Compute falls
        // back to DOD FMV and flags the fallback.
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = false;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000)); // DOD wins
        assert!(r.alternate_valuation_election_invalid_fallback);
        assert!(r.note.contains("invalid"));
    }

    #[test]
    fn alternate_valuation_election_without_av_fmv_falls_back() {
        // Election claimed but no AV FMV provided → can't apply §2032,
        // fall back to DOD with the invalid-election flag.
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = None;
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert!(r.alternate_valuation_election_invalid_fallback);
    }

    #[test]
    fn form_706_consistent_basis_cap_applied_when_form_below_fmv() {
        // §1014(f) — executor reported $80k on Form 706 but raw FMV was
        // $100k. Heir's basis capped at $80k (no claim higher than what
        // was reported for estate tax).
        let mut i = base();
        i.fmv_reported_on_form_706 = Some(dec!(80_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(80_000));
        assert!(r.form_706_consistent_basis_cap_applied);
        assert!(r.note.contains("§1014(f)"));
    }

    #[test]
    fn form_706_does_not_cap_when_form_above_fmv() {
        // §1014(f) is a ceiling, not a floor. If Form 706 reported
        // $120k but actual FMV at DOD was $100k, heir's basis stays at
        // $100k — the cap doesn't INCREASE basis.
        let mut i = base();
        i.fmv_reported_on_form_706 = Some(dec!(120_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert!(!r.form_706_consistent_basis_cap_applied);
    }

    #[test]
    fn ird_takes_precedence_over_alternate_valuation_election() {
        // §1014(c) IRD denies step-up entirely. Even with a valid §2032
        // election, an IRD asset (e.g., an IRA inside the estate) gets
        // no step-up. Ordering pinned because a future code path could
        // accidentally let §2032 override §1014(c).
        let mut i = base();
        i.is_income_in_respect_of_decedent = true;
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(10_000));
        assert_eq!(r.rule_applied, Section1014Rule::IncomeInRespectOfDecedent);
    }

    #[test]
    fn clawback_takes_precedence_over_alternate_valuation() {
        // Same ordering test for §1014(e). Even with a valid §2032
        // election, the clawback denies step-up.
        let mut i = base();
        i.decedent_received_as_gift_within_one_year = true;
        i.passes_back_to_donor_or_spouse = true;
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(10_000));
        assert_eq!(r.rule_applied, Section1014Rule::OneYearClawback);
    }

    #[test]
    fn zero_basis_full_step_up() {
        // Property with $0 basis (gift or fully-depreciated rental).
        // FMV at DOD = $500k → heir's basis = $500k, $500k gain
        // eliminated. The "die with $0 basis" strategy at its purest.
        let mut i = base();
        i.decedents_adjusted_basis = Decimal::ZERO;
        i.fmv_at_dod = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(500_000));
        assert_eq!(r.unrealized_gain_eliminated, dec!(500_000));
    }

    #[test]
    fn basis_equals_fmv_zero_gain_eliminated() {
        // No appreciation → no gain to eliminate, but the rule still
        // "applies" (it's just a no-op in dollar terms).
        let mut i = base();
        i.decedents_adjusted_basis = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(100_000));
        assert_eq!(r.unrealized_gain_eliminated, Decimal::ZERO);
        assert_eq!(r.rule_applied, Section1014Rule::StandardStepUp);
    }

    #[test]
    fn form_706_cap_with_alternate_valuation_compounds_correctly() {
        // Combined: §2032 picks alternate FMV $75k; §1014(f) caps at
        // $60k Form 706 → heir's basis = $60k. The cap applies AFTER
        // the §2032 selection, not before.
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        i.fmv_reported_on_form_706 = Some(dec!(60_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(60_000));
        assert_eq!(r.valuation_used, ValuationUsed::AlternateValuationDate);
        assert!(r.form_706_consistent_basis_cap_applied);
    }

    #[test]
    fn ird_with_form_706_cap_no_interaction() {
        // §1014(c) IRD returns decedent's basis directly; the Form 706
        // cap doesn't apply (because no step-up is happening anyway).
        // The cap_applied flag should stay false even if Form 706 was
        // reported low.
        let mut i = base();
        i.is_income_in_respect_of_decedent = true;
        i.fmv_reported_on_form_706 = Some(dec!(5_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(10_000));
        assert!(!r.form_706_consistent_basis_cap_applied);
    }

    #[test]
    fn note_describes_dod_valuation_when_no_election() {
        let r = compute(&base());
        assert!(r.note.contains("DOD"));
        assert!(r.note.contains("2026-03-15"));
        assert!(!r.note.contains("§2032"));
    }

    #[test]
    fn note_describes_alternate_valuation_when_elected() {
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(75_000));
        let r = compute(&i);
        assert!(r.note.contains("§2032"));
        assert!(r.note.contains("6 months after DOD"));
    }

    #[test]
    fn very_large_basis_no_precision_loss() {
        // Decimal arithmetic stays exact for arbitrary precision.
        // $1.234567 billion basis stepped up to $9.876543 billion FMV.
        let mut i = base();
        i.decedents_adjusted_basis = dec!(1_234_567_890.12);
        i.fmv_at_dod = dec!(9_876_543_210.98);
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(9_876_543_210.98));
        assert_eq!(r.unrealized_gain_eliminated, dec!(8_641_975_320.86));
    }

    #[test]
    fn alternate_valuation_step_down_works() {
        // §2032 elected with alternate FMV LOWER than DOD FMV. This is
        // the canonical reason executors elect — declining-asset estate.
        // DOD $100k → AV $60k → heir's basis = $60k.
        let mut i = base();
        i.alternate_valuation_election_made = true;
        i.election_lowers_both_gross_estate_and_tax = true;
        i.fmv_at_alternate_valuation_date = Some(dec!(60_000));
        let r = compute(&i);
        assert_eq!(r.heirs_basis, dec!(60_000));
        assert_eq!(r.unrealized_gain_eliminated, dec!(50_000));
    }
}
