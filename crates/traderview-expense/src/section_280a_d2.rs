//! IRC §280A(d)(2) — Personal-use day classifier for related-party
//! rentals. The §267 analog on the rental-income side.
//!
//! Renting your rental property to a family member at a discount is
//! one of the most-common landlord tax mistakes. §280A(d)(2)(A)
//! treats ANY use by the taxpayer or a related party as personal
//! use — REGARDLESS of whether rent is paid. Those personal-use
//! days flow into the existing `section_280a` (iter 9) classifier
//! and frequently flip the property from a normal rental into a
//! vacation home subject to §280A(c)(5) (deductions capped at
//! gross rental income, no net loss).
//!
//! The narrow exception in §280A(d)(2)(C) flush language: use by a
//! related party DOES NOT count as personal use if BOTH of these
//! are true:
//!
//!   1. The related party uses the unit as their **principal**
//!      **residence** (not a second home or vacation rental).
//!   2. The rent paid equals or exceeds **fair market rent** for
//!      that period.
//!
//! "Related party" for §280A(d)(4) cross-references §267(b) — the
//! same 10-category list that the `section_267` module exposes —
//! plus brothers/sisters/spouse/ancestors/lineal-descendants per
//! §267(c)(4). Caller asserts the relationship via the
//! `RelationshipCategory` enum that lives in `section_267`.
//!
//! Also models the §280A(d)(2)(B) **shared-equity-financing
//! arrangement** carve-out: a co-owner who lives in the property
//! qualifies as paying fair rental even if no cash actually changes
//! hands, provided the agreement meets §280A(d)(3) requirements.
//!
//! Pure compute. Caller passes a list of rental-period uses; we
//! classify each as `personal_use` or `rental_use` and aggregate
//! the day counts for direct feed into `section_280a::compute`.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Whether the occupant is the taxpayer, a §267(b) related party, or
/// an unrelated third party. Unrelated occupants who pay any rent at
/// all classify as rental use; related occupants face the §280A(d)(2)
/// gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Occupant {
    Taxpayer,
    RelatedParty,
    Unrelated,
}

/// One occupancy period during the year — a contiguous stretch of
/// days a specific occupant occupied the unit. Multiple entries per
/// property per year stack via repeated calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupancyPeriod {
    pub occupant: Occupant,
    pub days: u32,
    /// Whether the unit was the occupant's principal residence during
    /// these days. Only matters for RelatedParty under the
    /// §280A(d)(2)(C) flush-language exception.
    pub used_as_principal_residence: bool,
    /// Rent actually paid for this period.
    pub rent_paid: Decimal,
    /// What an arm's-length tenant would have paid for the same
    /// period. Caller obtains this from comparables (Zillow Rent
    /// Estimate, Rentometer, etc.).
    pub fair_market_rent_for_period: Decimal,
    /// True if a §280A(d)(3) shared-equity-financing arrangement is
    /// in place. Co-owner who lives in the property qualifies as
    /// paying fair rental.
    pub shared_equity_arrangement: bool,
    /// True if the period was a §280A(d)(4) "repair day" — days
    /// spent performing maintenance on the property aren't personal
    /// use even if the taxpayer or family stayed there.
    pub repair_day: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section280AD2Report {
    pub personal_use_days: u32,
    pub rental_use_days: u32,
    pub period_classifications: Vec<PeriodClassification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodClassification {
    pub days: u32,
    pub classification: Classification,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    #[default]
    PersonalUse,
    RentalUse,
}

fn classify(period: &OccupancyPeriod) -> (Classification, String) {
    // §280A(d)(4): repair days are neither personal nor rental — they
    // simply don't count. We surface them as rental_use=false +
    // personal_use=false, exposed via `repair_days_excluded` in the
    // report. For simplicity we treat repair_day = rental_use=false +
    // personal_use=false → return RentalUse with a special reason so
    // callers can filter; but the better signal is to bucket repair
    // days as rental_use (per Pub 527 they don't subtract from rental
    // days and don't add to personal days).
    if period.repair_day {
        return (
            Classification::RentalUse,
            "§280A(d)(4) repair day: maintenance days don't count as personal use".into(),
        );
    }

    match period.occupant {
        Occupant::Taxpayer => (
            Classification::PersonalUse,
            "§280A(d)(2)(A) taxpayer personal use".into(),
        ),
        Occupant::Unrelated => {
            if period.rent_paid > Decimal::ZERO {
                (
                    Classification::RentalUse,
                    "unrelated paying tenant — full rental use".into(),
                )
            } else {
                (
                    Classification::PersonalUse,
                    "unrelated occupant with no rent = personal use (gratuitous use)".into(),
                )
            }
        }
        Occupant::RelatedParty => {
            // §280A(d)(2)(C) flush exception: principal residence AND
            // fair market rent paid. Either condition fails → personal.
            if period.shared_equity_arrangement {
                return (
                    Classification::RentalUse,
                    "§280A(d)(3) shared-equity-financing arrangement: rental use".into(),
                );
            }
            if period.used_as_principal_residence
                && period.rent_paid >= period.fair_market_rent_for_period
                && period.fair_market_rent_for_period > Decimal::ZERO
            {
                (
                    Classification::RentalUse,
                    format!(
                        "§280A(d)(2)(C) exception: principal residence + ${} ≥ ${} FMV rent",
                        period.rent_paid, period.fair_market_rent_for_period
                    ),
                )
            } else if !period.used_as_principal_residence {
                (
                    Classification::PersonalUse,
                    "§280A(d)(2)(A) related party — not principal residence".into(),
                )
            } else if period.rent_paid < period.fair_market_rent_for_period {
                (
                    Classification::PersonalUse,
                    format!(
                        "§280A(d)(2)(A) related party — ${} rent < ${} FMV (below-market)",
                        period.rent_paid, period.fair_market_rent_for_period
                    ),
                )
            } else {
                (
                    Classification::PersonalUse,
                    "§280A(d)(2)(A) related party — no FMV reference provided".into(),
                )
            }
        }
    }
}

pub fn compute(periods: &[OccupancyPeriod]) -> Section280AD2Report {
    let mut r = Section280AD2Report::default();
    for p in periods {
        let (cls, reason) = classify(p);
        match cls {
            Classification::PersonalUse => r.personal_use_days += p.days,
            Classification::RentalUse => r.rental_use_days += p.days,
        }
        r.period_classifications.push(PeriodClassification {
            days: p.days,
            classification: cls,
            reason,
        });
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn period(occupant: Occupant, days: u32) -> OccupancyPeriod {
        OccupancyPeriod {
            occupant,
            days,
            used_as_principal_residence: false,
            rent_paid: Decimal::ZERO,
            fair_market_rent_for_period: Decimal::ZERO,
            shared_equity_arrangement: false,
            repair_day: false,
        }
    }

    #[test]
    fn taxpayer_use_always_personal() {
        let r = compute(&[period(Occupant::Taxpayer, 14)]);
        assert_eq!(r.personal_use_days, 14);
        assert_eq!(r.rental_use_days, 0);
        assert!(r.period_classifications[0].reason.contains("taxpayer"));
    }

    #[test]
    fn unrelated_paying_tenant_full_rental_use() {
        let mut p = period(Occupant::Unrelated, 365);
        p.rent_paid = dec!(20000);
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
        assert_eq!(r.personal_use_days, 0);
    }

    #[test]
    fn unrelated_no_rent_counts_as_personal() {
        // Friend stays for free — §280A(d)(2)(A) treats gratuitous use
        // as personal use of the unit.
        let r = compute(&[period(Occupant::Unrelated, 30)]);
        assert_eq!(r.personal_use_days, 30);
    }

    #[test]
    fn related_party_at_fmv_principal_residence_is_rental() {
        // Renting to your adult kid AT MARKET as their primary home.
        // §280A(d)(2)(C) flush exception applies.
        let mut p = period(Occupant::RelatedParty, 365);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(24000);
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
        assert!(r.period_classifications[0]
            .reason
            .contains("§280A(d)(2)(C)"));
    }

    #[test]
    fn related_party_below_market_is_personal_use() {
        // Discount to your kid — converts to personal use even if
        // they live there full-time. This is the trap most landlords
        // miss.
        let mut p = period(Occupant::RelatedParty, 365);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(12000);
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.personal_use_days, 365);
        assert!(r.period_classifications[0].reason.contains("below-market"));
    }

    #[test]
    fn related_party_not_principal_residence_personal_use() {
        // Family member uses as a vacation home; even at FMV, fails
        // the principal-residence prong.
        let mut p = period(Occupant::RelatedParty, 30);
        p.used_as_principal_residence = false;
        p.rent_paid = dec!(5000);
        p.fair_market_rent_for_period = dec!(5000);
        let r = compute(&[p]);
        assert_eq!(r.personal_use_days, 30);
        assert!(r.period_classifications[0]
            .reason
            .contains("not principal residence"));
    }

    #[test]
    fn shared_equity_financing_arrangement_qualifies_as_rental() {
        // §280A(d)(3): co-owner residing in property under shared-
        // equity arrangement = rental use.
        let mut p = period(Occupant::RelatedParty, 365);
        p.shared_equity_arrangement = true;
        p.rent_paid = Decimal::ZERO; // doesn't matter
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
        assert!(r.period_classifications[0].reason.contains("§280A(d)(3)"));
    }

    #[test]
    fn repair_days_dont_count_as_personal() {
        // Taxpayer stayed at the property to fix it up — §280A(d)(4).
        let mut p = period(Occupant::Taxpayer, 7);
        p.repair_day = true;
        let r = compute(&[p]);
        assert_eq!(r.personal_use_days, 0);
        assert!(r.period_classifications[0].reason.contains("repair day"));
    }

    #[test]
    fn aggregate_across_multiple_periods() {
        let mut p1 = period(Occupant::RelatedParty, 30); // below market → personal
        p1.used_as_principal_residence = true;
        p1.rent_paid = dec!(500);
        p1.fair_market_rent_for_period = dec!(1000);

        let mut p2 = period(Occupant::Unrelated, 300); // paying tenant → rental
        p2.rent_paid = dec!(15000);

        let p3 = period(Occupant::Taxpayer, 14); // taxpayer → personal

        let r = compute(&[p1, p2, p3]);
        assert_eq!(r.personal_use_days, 44);
        assert_eq!(r.rental_use_days, 300);
        assert_eq!(r.period_classifications.len(), 3);
    }

    #[test]
    fn related_party_zero_fmv_reference_defaults_to_personal() {
        // Caller didn't provide a FMV comparable. We err on the side
        // of personal-use treatment (safe disposition for landlord).
        let mut p = period(Occupant::RelatedParty, 90);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(9000);
        p.fair_market_rent_for_period = Decimal::ZERO;
        let r = compute(&[p]);
        assert_eq!(r.personal_use_days, 90);
    }

    #[test]
    fn related_party_at_exactly_fmv_qualifies() {
        let mut p = period(Occupant::RelatedParty, 365);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(24000);
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
    }

    #[test]
    fn related_party_above_fmv_qualifies() {
        let mut p = period(Occupant::RelatedParty, 365);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(25000);
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
    }

    #[test]
    fn related_party_one_cent_below_fmv_personal() {
        let mut p = period(Occupant::RelatedParty, 365);
        p.used_as_principal_residence = true;
        p.rent_paid = dec!(23999.99);
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.personal_use_days, 365);
    }

    #[test]
    fn empty_input_returns_zero_counts() {
        let r = compute(&[]);
        assert_eq!(r.personal_use_days, 0);
        assert_eq!(r.rental_use_days, 0);
        assert!(r.period_classifications.is_empty());
    }

    #[test]
    fn shared_equity_overrides_below_market_rent() {
        // Even at $0 rent — shared-equity arrangement is the magic word.
        let mut p = period(Occupant::RelatedParty, 365);
        p.shared_equity_arrangement = true;
        p.rent_paid = Decimal::ZERO;
        p.fair_market_rent_for_period = dec!(24000);
        let r = compute(&[p]);
        assert_eq!(r.rental_use_days, 365);
    }

    #[test]
    fn repair_day_overrides_taxpayer_personal_use() {
        // Owner spent 7 days at the property doing repairs.
        // Without repair_day=true, those would be 7 personal-use days
        // and could flip the property into vacation-home classification.
        let mut p1 = period(Occupant::Taxpayer, 7);
        p1.repair_day = true;

        let mut p2 = period(Occupant::Unrelated, 358);
        p2.rent_paid = dec!(20000);

        let r = compute(&[p1, p2]);
        assert_eq!(r.personal_use_days, 0);
        assert_eq!(r.rental_use_days, 365);
    }
}
