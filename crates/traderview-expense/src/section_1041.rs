//! IRC §1041 — Transfers of property between spouses or incident to
//! divorce.
//!
//! Completes the basis-transfer trio with `section_1014` (stepped-up
//! basis at death) and `section_1015` (lifetime gift carryover).
//! Defines property-division taxation for every divorce.
//!
//! **§1041(a)**: NO gain or loss is recognized on a transfer of
//! property between current spouses or between former spouses if the
//! transfer is "incident to divorce". The transferor doesn't pay tax on
//! the embedded appreciation.
//!
//! **§1041(b)**: The transferee takes the **transferor's adjusted
//! basis** as a carryover basis — regardless of whether basis is less
//! than, equal to, or greater than FMV at transfer. This is
//! distinguished from §1015 — there is **NO dual-basis rule** for
//! depreciated property between spouses. The loss-shifting concern that
//! drives §1015(a) doesn't apply because spouses are economically a
//! single unit. Holding period TACKS uniformly under §1223(2).
//!
//! **§1041(c) "incident to divorce" rules** per Treas. Reg.
//! § 1.1041-1T(b) Q&A-7:
//!
//! - **Within 1 year of marriage cessation** → **AUTOMATIC**. No further
//!   test required.
//! - **1 to 6 years post-cessation** → applies ONLY if made pursuant to
//!   a divorce or separation instrument. Without an instrument the
//!   presumption is against §1041 treatment.
//! - **After 6 years** → presumption against §1041 is strong; transfer
//!   qualifies only with explicit instrument language tying it to the
//!   divorce.
//!
//! **§1041(d) NR alien exception**: §1041 does NOT apply when the
//! transferee spouse (or former spouse) is a nonresident alien at the
//! time of transfer. The transferor recognizes immediate gain/loss as
//! though to a third party.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1041Input {
    pub transferor_adjusted_basis: Decimal,
    pub transferor_holding_period_start: NaiveDate,
    pub fmv_at_transfer: Decimal,
    pub transfer_date: NaiveDate,
    /// Date the marriage ended (divorce decree). `None` if the parties
    /// are still married at the time of transfer (then §1041
    /// automatically applies for currently-married transfers).
    pub marriage_cessation_date: Option<NaiveDate>,
    /// True if the transfer is made pursuant to a divorce or separation
    /// instrument. Drives the 1-6 year and >6 year tests under Treas.
    /// Reg. § 1.1041-1T(b).
    pub transfer_pursuant_to_divorce_instrument: bool,
    /// True if the transferee is a nonresident alien at the time of
    /// transfer — §1041(d) disqualification.
    pub transferee_is_nonresident_alien: bool,
    pub sale_price: Decimal,
    pub sale_date: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentRule {
    StillMarried,
    WithinOneYearAutomatic,
    OneToSixYearsPursuantToInstrument,
    BeyondSixYearsPursuantToInstrument,
    NotIncidentToDivorce,
    NonresidentAlienDisqualified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCharacter {
    ShortTermCapital,
    LongTermCapital,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1041Result {
    /// True if §1041(a) applies — no gain/loss at transfer; carryover basis.
    pub section_1041_applies: bool,
    pub incident_rule: IncidentRule,
    /// Gain/loss the transferor recognizes AT THE TIME OF TRANSFER.
    /// Zero when §1041 applies; FMV − basis when it doesn't.
    pub transferor_gain_loss_at_transfer: Decimal,
    /// Transferee's basis in the property. Transferor's carryover basis
    /// when §1041 applies; FMV at transfer otherwise.
    pub transferee_basis: Decimal,
    /// Gain or loss the transferee recognizes on subsequent sale.
    pub eventual_gain_loss_on_sale: Decimal,
    /// Holding-period start: transferor's acquisition (tacked) when
    /// §1041 applies; transfer date otherwise.
    pub holding_period_start: NaiveDate,
    pub holding_period_days: i64,
    pub character: CapitalCharacter,
    /// Days from marriage_cessation_date to transfer_date. `None` if
    /// still married.
    pub days_from_cessation_to_transfer: Option<i64>,
    pub note: String,
}

/// §1222 / §1223(2) holding-period boundary.
const ONE_YEAR_DAYS: i64 = 365;
/// §1041(c) 1-year automatic window.
const ONE_YEAR_BOUNDARY: i64 = 365;
/// §1041(c) 6-year presumption boundary.
const SIX_YEAR_BOUNDARY: i64 = 365 * 6;

pub fn compute(input: &Section1041Input) -> Section1041Result {
    // Step 1: NR alien disqualification (highest precedence).
    if input.transferee_is_nonresident_alien {
        return non_applicable_path(
            input,
            IncidentRule::NonresidentAlienDisqualified,
            "§1041(d) — transferee is a nonresident alien at time of transfer; §1041(a) does NOT apply; transferor recognizes immediate gain/loss",
        );
    }

    // Step 2: Classify under §1041(c) timing rule.
    let (rule, applies, days_since_cessation) = match input.marriage_cessation_date {
        None => (IncidentRule::StillMarried, true, None),
        Some(cessation) => {
            let days = (input.transfer_date - cessation).num_days();
            if days <= ONE_YEAR_BOUNDARY {
                (IncidentRule::WithinOneYearAutomatic, true, Some(days))
            } else if days <= SIX_YEAR_BOUNDARY {
                if input.transfer_pursuant_to_divorce_instrument {
                    (
                        IncidentRule::OneToSixYearsPursuantToInstrument,
                        true,
                        Some(days),
                    )
                } else {
                    (IncidentRule::NotIncidentToDivorce, false, Some(days))
                }
            } else {
                // > 6 years
                if input.transfer_pursuant_to_divorce_instrument {
                    (
                        IncidentRule::BeyondSixYearsPursuantToInstrument,
                        true,
                        Some(days),
                    )
                } else {
                    (IncidentRule::NotIncidentToDivorce, false, Some(days))
                }
            }
        }
    };

    if !applies {
        let note = match rule {
            IncidentRule::NotIncidentToDivorce => format!(
                "§1041 does NOT apply — transfer {} days after marriage cessation, no divorce instrument; transferor recognizes immediate gain/loss",
                days_since_cessation.unwrap_or(0)
            ),
            _ => "§1041 disqualified".to_string(),
        };
        let mut result = non_applicable_path(input, rule, &note);
        result.days_from_cessation_to_transfer = days_since_cessation;
        return result;
    }

    // Step 3: §1041 applies → no gain/loss at transfer, basis carries.
    let transferee_basis = input.transferor_adjusted_basis;
    let eventual_gain = input.sale_price - transferee_basis;
    let holding_days =
        (input.sale_date - input.transferor_holding_period_start).num_days();
    let character = if holding_days > ONE_YEAR_DAYS {
        CapitalCharacter::LongTermCapital
    } else {
        CapitalCharacter::ShortTermCapital
    };

    let rule_label = match rule {
        IncidentRule::StillMarried => "current spouses",
        IncidentRule::WithinOneYearAutomatic => {
            "within 1 year of cessation — automatic"
        }
        IncidentRule::OneToSixYearsPursuantToInstrument => {
            "1-6 years post-cessation, pursuant to divorce instrument"
        }
        IncidentRule::BeyondSixYearsPursuantToInstrument => {
            "beyond 6 years, pursuant to divorce instrument (presumption-rebutting)"
        }
        _ => "incident to divorce",
    };
    let note = format!(
        "§1041(a) applies ({}); transferor recognizes $0; transferee takes ${} carryover basis; §1223(2) holding period tacks from {}; sale gain/loss = ${}",
        rule_label,
        transferee_basis.round_dp(2),
        input.transferor_holding_period_start,
        eventual_gain.round_dp(2)
    );

    Section1041Result {
        section_1041_applies: true,
        incident_rule: rule,
        transferor_gain_loss_at_transfer: Decimal::ZERO,
        transferee_basis,
        eventual_gain_loss_on_sale: eventual_gain,
        holding_period_start: input.transferor_holding_period_start,
        holding_period_days: holding_days,
        character,
        days_from_cessation_to_transfer: days_since_cessation,
        note,
    }
}

fn non_applicable_path(
    input: &Section1041Input,
    rule: IncidentRule,
    note_prefix: &str,
) -> Section1041Result {
    // §1041 doesn't apply — transferor recognizes gain/loss; transferee
    // takes FMV basis; holding period starts at transfer.
    let transferor_gain = input.fmv_at_transfer - input.transferor_adjusted_basis;
    let transferee_basis = input.fmv_at_transfer;
    let eventual_gain = input.sale_price - transferee_basis;
    let holding_days = (input.sale_date - input.transfer_date).num_days();
    let character = if holding_days > ONE_YEAR_DAYS {
        CapitalCharacter::LongTermCapital
    } else {
        CapitalCharacter::ShortTermCapital
    };
    Section1041Result {
        section_1041_applies: false,
        incident_rule: rule,
        transferor_gain_loss_at_transfer: transferor_gain,
        transferee_basis,
        eventual_gain_loss_on_sale: eventual_gain,
        holding_period_start: input.transfer_date,
        holding_period_days: holding_days,
        character,
        days_from_cessation_to_transfer: None,
        note: note_prefix.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section1041Input {
        Section1041Input {
            transferor_adjusted_basis: dec!(10_000),
            transferor_holding_period_start: d(2020, 1, 1),
            fmv_at_transfer: dec!(100_000),
            transfer_date: d(2026, 1, 1),
            marriage_cessation_date: None, // currently married
            transfer_pursuant_to_divorce_instrument: false,
            transferee_is_nonresident_alien: false,
            sale_price: dec!(150_000),
            sale_date: d(2026, 7, 1),
        }
    }

    #[test]
    fn current_spouses_section_1041_applies() {
        // No cessation date → still married → §1041 applies.
        let r = compute(&base());
        assert!(r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::StillMarried);
        assert_eq!(r.transferor_gain_loss_at_transfer, Decimal::ZERO);
        assert_eq!(r.transferee_basis, dec!(10_000));
    }

    #[test]
    fn within_one_year_automatic() {
        // Cessation 6 months before transfer → within 1 year window.
        let mut i = base();
        i.marriage_cessation_date = Some(d(2025, 7, 1));
        let r = compute(&i);
        assert!(r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::WithinOneYearAutomatic);
        assert_eq!(r.days_from_cessation_to_transfer, Some(184));
    }

    #[test]
    fn within_one_year_exact_boundary_day_365_automatic() {
        // Day 365 from cessation = within 1 year (≤ 365 days).
        let mut i = base();
        i.marriage_cessation_date = Some(d(2025, 1, 1));
        i.transfer_date = d(2026, 1, 1);
        let r = compute(&i);
        assert!(r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::WithinOneYearAutomatic);
        assert_eq!(r.days_from_cessation_to_transfer, Some(365));
    }

    #[test]
    fn day_366_requires_divorce_instrument() {
        // Day 366 from cessation: no longer automatic. Without
        // instrument flag → §1041 does NOT apply.
        let mut i = base();
        i.marriage_cessation_date = Some(d(2025, 1, 1));
        i.transfer_date = d(2026, 1, 2); // 366 days
        i.transfer_pursuant_to_divorce_instrument = false;
        let r = compute(&i);
        assert!(!r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::NotIncidentToDivorce);
    }

    #[test]
    fn day_366_with_instrument_applies() {
        // Day 366 WITH instrument flag → §1041 applies under 1-6 year
        // rule.
        let mut i = base();
        i.marriage_cessation_date = Some(d(2025, 1, 1));
        i.transfer_date = d(2026, 1, 2);
        i.transfer_pursuant_to_divorce_instrument = true;
        let r = compute(&i);
        assert!(r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::OneToSixYearsPursuantToInstrument);
    }

    #[test]
    fn six_year_exact_boundary_with_instrument_applies() {
        // 6 × 365 = 2190 days. Day 2190 = within 6-year boundary.
        // 2020-01-01 + 2190 days = 2025-12-30 (2020 + 2024 leap years
        // add up to make 2025-12-31 land at day 2191, so day 2190 is
        // 2025-12-30).
        let mut i = base();
        i.marriage_cessation_date = Some(d(2020, 1, 1));
        i.transfer_date = d(2025, 12, 30);
        i.transfer_pursuant_to_divorce_instrument = true;
        let r = compute(&i);
        assert_eq!(r.days_from_cessation_to_transfer, Some(2190));
        assert!(r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::OneToSixYearsPursuantToInstrument);
    }

    #[test]
    fn day_2191_beyond_six_years_with_instrument_applies() {
        // Day 2191 = > 6 × 365 → BeyondSixYears rule fires.
        let mut i = base();
        i.marriage_cessation_date = Some(d(2020, 1, 1));
        i.transfer_date = d(2025, 12, 31); // day 2191
        i.transfer_pursuant_to_divorce_instrument = true;
        let r = compute(&i);
        assert_eq!(r.days_from_cessation_to_transfer, Some(2191));
        assert_eq!(r.incident_rule, IncidentRule::BeyondSixYearsPursuantToInstrument);
        assert!(r.section_1041_applies);
    }

    #[test]
    fn beyond_six_years_without_instrument_disqualified() {
        // Day 2191 without instrument → §1041 doesn't apply.
        let mut i = base();
        i.marriage_cessation_date = Some(d(2020, 1, 1));
        i.transfer_date = d(2026, 1, 1);
        i.transfer_pursuant_to_divorce_instrument = false;
        let r = compute(&i);
        assert!(!r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::NotIncidentToDivorce);
    }

    #[test]
    fn nonresident_alien_spouse_disqualifies_section_1041() {
        // §1041(d). NR alien → §1041 doesn't apply. Highest precedence
        // (fires before timing rule check).
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        let r = compute(&i);
        assert!(!r.section_1041_applies);
        assert_eq!(r.incident_rule, IncidentRule::NonresidentAlienDisqualified);
    }

    #[test]
    fn nonresident_alien_overrides_otherwise_applicable_path() {
        // Even when still married + NR alien, §1041 doesn't apply.
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        i.marriage_cessation_date = None;
        let r = compute(&i);
        assert_eq!(r.incident_rule, IncidentRule::NonresidentAlienDisqualified);
    }

    #[test]
    fn carryover_basis_when_applies_no_gain_at_transfer() {
        // Transferor basis $10k, FMV $100k. Under §1041: transferor
        // recognizes 0 gain; transferee takes $10k basis (not $100k FMV).
        let r = compute(&base());
        assert_eq!(r.transferee_basis, dec!(10_000));
        assert_eq!(r.transferor_gain_loss_at_transfer, Decimal::ZERO);
    }

    #[test]
    fn fmv_basis_when_section_1041_does_not_apply() {
        // NR alien path: transferor recognizes FMV − basis = $90k gain;
        // transferee takes FMV ($100k) basis.
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        let r = compute(&i);
        assert_eq!(r.transferor_gain_loss_at_transfer, dec!(90_000));
        assert_eq!(r.transferee_basis, dec!(100_000));
    }

    #[test]
    fn sale_uses_carryover_basis_when_applies() {
        // Sale $150k − transferee basis $10k = $140k eventual gain.
        let r = compute(&base());
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(140_000));
    }

    #[test]
    fn sale_uses_fmv_basis_when_section_1041_disqualified() {
        // Sale $150k − transferee FMV basis $100k = $50k eventual gain.
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        let r = compute(&i);
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(50_000));
    }

    #[test]
    fn holding_period_tacks_when_section_1041_applies() {
        // Transferor held since 2020-01-01; sale 2026-07-01 = 6.5 years.
        let r = compute(&base());
        assert_eq!(r.holding_period_start, d(2020, 1, 1));
        assert!(r.holding_period_days > ONE_YEAR_DAYS);
        assert_eq!(r.character, CapitalCharacter::LongTermCapital);
    }

    #[test]
    fn holding_period_starts_at_transfer_when_disqualified() {
        // NR alien path: holding period starts at transfer date, not
        // transferor's acquisition. Tacking only applies when §1041 does.
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        let r = compute(&i);
        assert_eq!(r.holding_period_start, d(2026, 1, 1));
        assert_eq!(r.holding_period_days, 181); // Jan 1 → Jul 1 ≈ 6 mo
        assert_eq!(r.character, CapitalCharacter::ShortTermCapital);
    }

    #[test]
    fn depreciated_property_no_dual_basis_unlike_section_1015() {
        // Distinguishes §1041 from §1015. Donor basis $100k, FMV $50k
        // at transfer. Under §1015 there's a dual-basis rule. Under
        // §1041 — even on depreciated property — the transferee takes
        // the transferor's basis for ALL purposes, gain OR loss.
        let mut i = base();
        i.transferor_adjusted_basis = dec!(100_000);
        i.fmv_at_transfer = dec!(50_000);
        i.sale_price = dec!(30_000); // sale at loss
        let r = compute(&i);
        assert!(r.section_1041_applies);
        assert_eq!(r.transferee_basis, dec!(100_000)); // donor's basis, not FMV
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(-70_000)); // loss recognized
    }

    #[test]
    fn depreciated_property_recognized_loss_at_carryover_basis() {
        // Same setup as above but sale price higher than FMV. Loss
        // would be smaller. Still uses transferor basis (no phantom
        // zone like §1015).
        let mut i = base();
        i.transferor_adjusted_basis = dec!(100_000);
        i.fmv_at_transfer = dec!(50_000);
        i.sale_price = dec!(75_000);
        let r = compute(&i);
        assert!(r.section_1041_applies);
        assert_eq!(r.transferee_basis, dec!(100_000));
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(-25_000));
    }

    #[test]
    fn zero_basis_transferor_zero_basis_transferee() {
        let mut i = base();
        i.transferor_adjusted_basis = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.transferee_basis, Decimal::ZERO);
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(150_000));
    }

    #[test]
    fn note_describes_each_incident_rule_path() {
        // Each IncidentRule path should produce a distinct human-
        // readable description so the downstream UI can show the
        // taxpayer which rule actually applied.
        let still_married = compute(&base());
        assert!(still_married.note.contains("current spouses"));

        let mut within_one = base();
        within_one.marriage_cessation_date = Some(d(2025, 7, 1));
        let r = compute(&within_one);
        assert!(r.note.contains("within 1 year"));

        let mut one_to_six = base();
        one_to_six.marriage_cessation_date = Some(d(2024, 1, 1));
        one_to_six.transfer_pursuant_to_divorce_instrument = true;
        let r = compute(&one_to_six);
        assert!(r.note.contains("1-6 years"));

        let mut beyond_six = base();
        beyond_six.marriage_cessation_date = Some(d(2019, 1, 1));
        beyond_six.transfer_pursuant_to_divorce_instrument = true;
        let r = compute(&beyond_six);
        assert!(r.note.contains("beyond 6 years"));
    }

    #[test]
    fn note_for_disqualified_paths_explains_why() {
        let mut i = base();
        i.transferee_is_nonresident_alien = true;
        let r = compute(&i);
        assert!(r.note.contains("§1041(d)"));
        assert!(r.note.contains("nonresident alien"));

        let mut not_incident = base();
        not_incident.marriage_cessation_date = Some(d(2020, 1, 1));
        not_incident.transfer_date = d(2027, 1, 1); // > 6 years
        not_incident.transfer_pursuant_to_divorce_instrument = false;
        let r = compute(&not_incident);
        assert!(r.note.contains("does NOT apply"));
    }

    #[test]
    fn very_large_basis_no_precision_loss() {
        // Multi-billion dollar HNW divorce transfer. Decimal must stay
        // exact.
        let mut i = base();
        i.transferor_adjusted_basis = dec!(1_234_567_890.12);
        i.fmv_at_transfer = dec!(10_000_000_000);
        i.sale_price = dec!(15_000_000_000);
        let r = compute(&i);
        assert_eq!(r.transferee_basis, dec!(1_234_567_890.12));
        assert_eq!(r.eventual_gain_loss_on_sale, dec!(13_765_432_109.88));
    }

    #[test]
    fn one_year_boundary_366_days_post_holding_period() {
        // §1041 applies + transferor held since 2025-06-30 + sale day
        // 2026-07-01 = 366 days total holding tacked → LTCG.
        let mut i = base();
        i.transferor_holding_period_start = d(2025, 6, 30);
        i.sale_date = d(2026, 7, 1);
        let r = compute(&i);
        assert_eq!(r.holding_period_days, 366);
        assert_eq!(r.character, CapitalCharacter::LongTermCapital);
    }

    #[test]
    fn one_year_boundary_365_days_short_term() {
        let mut i = base();
        i.transferor_holding_period_start = d(2025, 7, 1);
        i.sale_date = d(2026, 7, 1);
        let r = compute(&i);
        assert_eq!(r.holding_period_days, 365);
        assert_eq!(r.character, CapitalCharacter::ShortTermCapital);
    }
}
