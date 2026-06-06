//! IRC §1031(f) — Special rules for exchanges between related persons.
//!
//! §1031(a) defers gain on a like-kind exchange of business or
//! investment property — iter 7's `disposition.rs` covers the base
//! case. §1031(f) adds an anti-abuse rule that bites landlords
//! swapping property with family members or controlled entities:
//! the deferred gain is **recognized retroactively** if either party
//! disposes of the property received within **two years** of the
//! exchange.
//!
//! §1031(f)(1)(C) — the disposition triggers gain recognition in the
//! year of the disqualifying disposition (not the original exchange
//! year). The character is the same as it would have been at the
//! original exchange.
//!
//! §1031(f)(2) **exceptions** — disqualification does NOT apply when:
//!
//!   1. **Death** of either party (§1031(f)(2)(A)).
//!   2. **Involuntary conversion** of the property under §1033
//!      (§1031(f)(2)(B)).
//!   3. **Lack of tax-avoidance purpose** — the taxpayer can establish
//!      to the satisfaction of the Secretary that neither the
//!      exchange nor the subsequent disposition had as one of its
//!      principal purposes the avoidance of federal income tax
//!      (§1031(f)(2)(C)).
//!
//! §1031(f)(4) extends the rule to indirect exchanges arranged through
//! qualified intermediaries when one of the principal purposes is to
//! escape §1031(f) — caller's responsibility to assert this when
//! applicable.
//!
//! Related-party determination uses the §267(b) categories from
//! `section_267::RelationshipCategory`. §1031(f)(3) cross-references
//! §267(b) (and §707(b)(1) for partnerships).
//!
//! Pure compute. Caller asserts the relationship + dates + optional
//! exception; we evaluate the 2-year window and the retroactive-
//! recognition outcome.

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use crate::section_267::RelationshipCategory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisqualifyingExceptionReason {
    /// §1031(f)(2)(A) — death of either party.
    DeathOfParty,
    /// §1031(f)(2)(B) — involuntary conversion under §1033.
    InvoluntaryConversion,
    /// §1031(f)(2)(C) — lack of tax-avoidance purpose, established to
    /// IRS satisfaction.
    LackOfTaxAvoidancePurpose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GainCharacter {
    #[default]
    None,
    LongTermCapitalGain,
    ShortTermCapitalGain,
    /// §1250 unrecaptured (depreciation recapture).
    Section1250Unrecaptured,
    /// §1245 ordinary recapture for personal property.
    Section1245OrdinaryRecapture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1031FInput {
    pub deferred_gain_at_exchange: Decimal,
    pub original_exchange_date: NaiveDate,
    pub relationship: RelationshipCategory,
    /// Date the property received in the §1031 exchange was disposed
    /// of by either party. `None` when no subsequent disposition has
    /// occurred yet.
    pub subsequent_disposition_date: Option<NaiveDate>,
    /// §1031(f)(2) exception, if any.
    pub exception: Option<DisqualifyingExceptionReason>,
    /// Character of the originally-deferred gain — used to label what
    /// gets recognized retroactively.
    pub deferred_gain_character: GainCharacter,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1031FResult {
    pub is_related_party_exchange: bool,
    pub two_year_window_end: Option<NaiveDate>,
    pub days_to_window_end: Option<i64>,
    pub window_still_open: bool,
    pub disqualifying_disposition_triggered: bool,
    pub gain_recognized_retroactively: Decimal,
    pub recognized_in_year: Option<i32>,
    pub character: GainCharacter,
    pub exception_applied: Option<DisqualifyingExceptionReason>,
    pub note: String,
}

fn two_year_window_end(exchange_date: NaiveDate) -> Option<NaiveDate> {
    exchange_date.checked_add_months(chrono::Months::new(24))
}

pub fn compute(input: &Section1031FInput) -> Section1031FResult {
    let mut r = Section1031FResult {
        is_related_party_exchange: input.relationship.is_related(),
        two_year_window_end: two_year_window_end(input.original_exchange_date),
        ..Section1031FResult::default()
    };

    if !r.is_related_party_exchange {
        r.note = "unrelated parties — §1031(f) does not apply".into();
        return r;
    }

    if input.deferred_gain_at_exchange <= Decimal::ZERO {
        r.note = "no deferred gain — §1031(f) clawback has nothing to recognize".into();
        return r;
    }

    // Compute days remaining in the 2-year window from the disposition
    // (or today if no disposition yet — caller is asking about exposure).
    if let (Some(window_end), Some(disposition)) =
        (r.two_year_window_end, input.subsequent_disposition_date)
    {
        r.days_to_window_end = Some((window_end - disposition).num_days());
        r.window_still_open = disposition < window_end;
    } else if let Some(window_end) = r.two_year_window_end {
        // No disposition yet — report exposure window from exchange date.
        let today = chrono::Utc::now().date_naive();
        r.days_to_window_end = Some((window_end - today).num_days());
        r.window_still_open = today < window_end;
    }

    let disposition = match input.subsequent_disposition_date {
        Some(d) => d,
        None => {
            // No disposition yet — window is still open if we're within
            // 2 years, otherwise the §1031 deferral has matured cleanly.
            r.note = if r.window_still_open {
                format!(
                    "related-party exchange — §1031(f) window open until {} ({} days remaining). No disposition yet.",
                    r.two_year_window_end.map(|d| d.to_string()).unwrap_or_default(),
                    r.days_to_window_end.unwrap_or_default(),
                )
            } else {
                "related-party exchange — §1031(f) 2-year window matured cleanly; no clawback exposure remaining.".into()
            };
            return r;
        }
    };

    let within_window = match r.two_year_window_end {
        Some(end) => disposition < end,
        None => false,
    };

    if !within_window {
        r.note = format!(
            "disposition on {} is AFTER 2-year window ending {} — §1031(f) deferral preserved",
            disposition,
            r.two_year_window_end
                .map(|d| d.to_string())
                .unwrap_or_default()
        );
        return r;
    }

    // Within window — check exceptions.
    if let Some(reason) = input.exception {
        r.exception_applied = Some(reason);
        r.note = format!(
            "disposition within 2-year window BUT §1031(f)(2) exception applies ({:?}); no retroactive recognition",
            reason
        );
        return r;
    }

    // Triggered: retroactive recognition in disposition year.
    r.disqualifying_disposition_triggered = true;
    r.gain_recognized_retroactively = input.deferred_gain_at_exchange;
    r.recognized_in_year = Some(disposition.year());
    r.character = input.deferred_gain_character;
    r.note = format!(
        "§1031(f)(1)(C) disqualifying disposition on {} within 2-year window ending {}: ${} {:?} recognized retroactively in tax year {}",
        disposition,
        r.two_year_window_end.map(|d| d.to_string()).unwrap_or_default(),
        r.gain_recognized_retroactively,
        r.character,
        disposition.year(),
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn base() -> Section1031FInput {
        Section1031FInput {
            deferred_gain_at_exchange: dec!(100000),
            original_exchange_date: date(2023, 6, 1),
            relationship: RelationshipCategory::FamilyMember,
            subsequent_disposition_date: None,
            exception: None,
            deferred_gain_character: GainCharacter::LongTermCapitalGain,
        }
    }

    #[test]
    fn unrelated_parties_section_does_not_apply() {
        let mut i = base();
        i.relationship = RelationshipCategory::Unrelated;
        let r = compute(&i);
        assert!(!r.is_related_party_exchange);
        assert!(!r.disqualifying_disposition_triggered);
        assert!(r.note.contains("does not apply"));
    }

    #[test]
    fn family_disposition_within_2_year_window_triggers() {
        // Exchange 2023-06-01, disposition 2024-12-01 (~18 months later).
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        let r = compute(&i);
        assert!(r.disqualifying_disposition_triggered);
        assert_eq!(r.gain_recognized_retroactively, dec!(100000));
        assert_eq!(r.recognized_in_year, Some(2024));
        assert_eq!(r.character, GainCharacter::LongTermCapitalGain);
    }

    #[test]
    fn disposition_after_2_year_window_preserves_deferral() {
        // Window ends 2025-06-01. Disposition 2025-07-01 = clean.
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2025, 7, 1));
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
        assert!(r.note.contains("AFTER 2-year window"));
    }

    #[test]
    fn disposition_exactly_at_window_end_preserves_deferral() {
        // Window end is exclusive — disposition ON the end date is fine.
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2025, 6, 1));
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
    }

    #[test]
    fn disposition_one_day_before_window_end_triggers() {
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2025, 5, 31));
        let r = compute(&i);
        assert!(r.disqualifying_disposition_triggered);
    }

    #[test]
    fn death_exception_blocks_recognition() {
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        i.exception = Some(DisqualifyingExceptionReason::DeathOfParty);
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
        assert_eq!(
            r.exception_applied,
            Some(DisqualifyingExceptionReason::DeathOfParty)
        );
    }

    #[test]
    fn involuntary_conversion_exception_blocks_recognition() {
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        i.exception = Some(DisqualifyingExceptionReason::InvoluntaryConversion);
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
    }

    #[test]
    fn lack_of_tax_avoidance_purpose_exception_blocks_recognition() {
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        i.exception = Some(DisqualifyingExceptionReason::LackOfTaxAvoidancePurpose);
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
        assert!(r.note.contains("§1031(f)(2)"));
    }

    #[test]
    fn no_disposition_within_window_reports_exposure_open() {
        // Exchange 2025-06-01 — well in the future when run today,
        // so the window is "open" with positive days remaining.
        let mut i = base();
        i.original_exchange_date = date(2099, 6, 1); // forced future
        let r = compute(&i);
        assert!(r.window_still_open);
        assert!(r.days_to_window_end.unwrap() > 0);
        assert!(r.note.contains("window open"));
    }

    #[test]
    fn no_disposition_after_window_matured_cleanly() {
        // Exchange 2010-01-01 — window long since closed; clean.
        let mut i = base();
        i.original_exchange_date = date(2010, 1, 1);
        let r = compute(&i);
        assert!(!r.window_still_open);
        assert!(r.note.contains("matured cleanly"));
    }

    #[test]
    fn zero_deferred_gain_no_clawback() {
        let mut i = base();
        i.deferred_gain_at_exchange = Decimal::ZERO;
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
        assert!(r.note.contains("no deferred gain"));
    }

    #[test]
    fn all_267b_categories_trigger_when_disposition_within_window() {
        for rel in [
            RelationshipCategory::FamilyMember,
            RelationshipCategory::IndividualAndControlledCorp,
            RelationshipCategory::TwoControlledCorps,
            RelationshipCategory::GrantorAndTrustFiduciary,
            RelationshipCategory::TwoTrustFiduciariesSameGrantor,
            RelationshipCategory::TrustFiduciaryAndBeneficiary,
            RelationshipCategory::TrustFiduciaryAndOtherBeneficiary,
            RelationshipCategory::CorpAndPartnershipCommonOwner,
            RelationshipCategory::TwoSCorps,
            RelationshipCategory::EstateExecutorAndBeneficiary,
        ] {
            let mut i = base();
            i.relationship = rel;
            i.subsequent_disposition_date = Some(date(2024, 12, 1));
            let r = compute(&i);
            assert!(r.disqualifying_disposition_triggered, "{:?}", rel);
        }
    }

    #[test]
    fn character_preserved_when_triggered() {
        // §1031(f)(1)(C) preserves the gain's original character. The
        // module just echoes whatever the caller asserted.
        let mut i = base();
        i.deferred_gain_character = GainCharacter::Section1250Unrecaptured;
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        let r = compute(&i);
        assert_eq!(r.character, GainCharacter::Section1250Unrecaptured);
    }

    #[test]
    fn two_year_window_end_uses_calendar_months_not_days() {
        // Exchange Feb 29 2024 (leap day). Window ends Feb 28 2026
        // (or Feb 29 2026 if 2026 were leap; it's not).
        let mut i = base();
        i.original_exchange_date = date(2024, 2, 29);
        let r = compute(&i);
        // chrono's checked_add_months adjusts Feb 29 + 24mo → Feb 28 2026.
        assert_eq!(r.two_year_window_end, Some(date(2026, 2, 28)));
    }

    #[test]
    fn related_with_exception_within_window_logs_exception_in_note() {
        let mut i = base();
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        i.exception = Some(DisqualifyingExceptionReason::InvoluntaryConversion);
        let r = compute(&i);
        assert!(r.note.contains("InvoluntaryConversion") || r.note.contains("§1031(f)(2)"));
    }

    #[test]
    fn unrelated_disposition_within_window_no_trigger() {
        // Sanity: unrelated parties get NO clawback even within window.
        let mut i = base();
        i.relationship = RelationshipCategory::Unrelated;
        i.subsequent_disposition_date = Some(date(2024, 12, 1));
        let r = compute(&i);
        assert!(!r.disqualifying_disposition_triggered);
    }

    #[test]
    fn recognized_in_year_matches_disposition_year_not_exchange_year() {
        // §1031(f)(1)(C): retroactive recognition in DISPOSITION year,
        // not the exchange year.
        let mut i = base();
        i.original_exchange_date = date(2022, 6, 1);
        i.subsequent_disposition_date = Some(date(2024, 1, 15));
        let r = compute(&i);
        assert!(r.disqualifying_disposition_triggered);
        assert_eq!(r.recognized_in_year, Some(2024));
    }
}
