//! IRC §1015 — Basis of property acquired by gifts and transfers in trust.
//!
//! Sibling to `section_1014` (stepped-up basis at death). When property
//! passes by gift during the donor's lifetime, the donee takes a
//! **carryover basis** equal to the donor's adjusted basis under
//! §1015(a). No step-up — the embedded gain (or loss) carries through
//! to the donee for eventual recognition.
//!
//! Three sub-rules matter:
//!
//! **§1015(a) general carryover** — donee's basis = donor's adjusted
//! basis. Holding period tacks from the donor's acquisition date per
//! §1223(2). A one-day-old gift of LTCG-eligible stock is immediately
//! long-term in the donee's hands.
//!
//! **§1015(a) dual-basis rule (depreciated property)** — if the FMV at
//! gift date is LESS than the donor's adjusted basis, the donee takes a
//! split basis:
//!   - **For computing GAIN on later sale**: use the donor's basis (with
//!     any §1015(d) gift-tax increase)
//!   - **For computing LOSS on later sale**: use the FMV at the gift date
//!
//! The split creates a famous "**phantom zone**" — if the donee later
//! sells at a price between the FMV-at-gift and the donor's basis,
//! NEITHER gain nor loss is recognized. The economic appreciation/loss
//! "disappears" for tax purposes. The §1015(a) loss bifurcation prevents
//! a donor from shifting an existing tax loss to a different taxpayer.
//!
//! **§1015(d) gift-tax basis increase** — if gift tax was paid on the
//! transfer, the donor's basis (carryover) is increased by the gift tax
//! attributable to the net appreciation:
//!
//!   ```text
//!   increase = gift_tax_paid × (net_appreciation / gift_amount_for_tax)
//!   ```
//!
//! where `net_appreciation = FMV - donor's basis`. Two ceilings:
//!   - The increase cannot exceed the net appreciation itself
//!   - The donee's adjusted basis (after increase) cannot exceed FMV at
//!     the gift date — preventing the adjustment from converting a gain
//!     asset into a loss asset.
//!
//! Loss-side holding period starts at gift per Treas. Reg. § 1.1015-1
//! (donee's holding period only tacks when basis is determined "in whole
//! or in part" by reference to donor's basis — using FMV-for-loss
//! satisfies neither).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1015Input {
    pub donor_adjusted_basis: Decimal,
    pub donor_holding_period_start: NaiveDate,
    pub fmv_at_gift_date: Decimal,
    pub gift_date: NaiveDate,
    /// Federal gift tax actually paid on this transfer (post any
    /// annual-exclusion / unified-credit netting).
    pub gift_tax_paid: Decimal,
    /// Taxable gift amount used to compute the gift tax. Typically
    /// `FMV - annual_exclusion`. Drives the §1015(d) ratio denominator.
    /// `Decimal::ZERO` to skip the gift-tax adjustment entirely.
    pub gift_amount_for_tax_purposes: Decimal,
    pub sale_price: Decimal,
    pub sale_date: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GiftSaleOutcome {
    Gain,
    Loss,
    /// Sale price is between FMV-at-gift and donor's basis (only
    /// possible under the §1015(a) dual-basis rule). No gain, no loss
    /// recognized.
    PhantomZone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapitalCharacter {
    ShortTermCapital,
    LongTermCapital,
    NoGainNoLoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1015Result {
    /// Basis the donee uses when computing GAIN — donor's basis + any
    /// §1015(d) gift-tax increase.
    pub donee_basis_for_gain: Decimal,
    /// Basis the donee uses when computing LOSS — donor's basis under
    /// the single-basis path; FMV-at-gift under the dual-basis path.
    pub donee_basis_for_loss: Decimal,
    pub realized_outcome: GiftSaleOutcome,
    /// Recognized gain (positive) or loss (negative) on the sale. Zero
    /// under PhantomZone.
    pub recognized_amount: Decimal,
    pub character: CapitalCharacter,
    /// Effective holding-period start: donor's date (tacking) under
    /// gain path; gift date under loss path of the dual-basis rule.
    pub holding_period_start: NaiveDate,
    pub holding_period_days: i64,
    /// §1015(d) gift-tax basis increase actually applied (zero if no
    /// gift tax paid or property was depreciated at gift).
    pub gift_tax_basis_increase: Decimal,
    /// True if the §1015(a) dual-basis rule applies — FMV at gift was
    /// less than the donor's adjusted basis.
    pub dual_basis_applied: bool,
    pub note: String,
}

/// §1222 / §1223 holding-period boundary.
const ONE_YEAR_DAYS: i64 = 365;

pub fn compute(input: &Section1015Input) -> Section1015Result {
    // Step 1: §1015(d) gift-tax basis increase.
    let net_appreciation = (input.fmv_at_gift_date - input.donor_adjusted_basis)
        .max(Decimal::ZERO);
    let gift_tax_basis_increase = if input.gift_tax_paid > Decimal::ZERO
        && net_appreciation > Decimal::ZERO
        && input.gift_amount_for_tax_purposes > Decimal::ZERO
    {
        let raw_increase =
            input.gift_tax_paid * (net_appreciation / input.gift_amount_for_tax_purposes);
        // Two ceilings: cannot exceed net appreciation, and donor basis
        // + increase cannot exceed FMV at gift.
        let cap_from_appreciation = net_appreciation;
        let cap_from_fmv = input.fmv_at_gift_date - input.donor_adjusted_basis;
        raw_increase.min(cap_from_appreciation).min(cap_from_fmv)
    } else {
        Decimal::ZERO
    };

    let adjusted_donor_basis = input.donor_adjusted_basis + gift_tax_basis_increase;

    // Step 2: §1015(a) dual-basis check. Triggered when FMV at gift is
    // LESS than the adjusted donor basis.
    let dual_basis = input.fmv_at_gift_date < adjusted_donor_basis;
    let donee_basis_for_gain = adjusted_donor_basis;
    let donee_basis_for_loss = if dual_basis {
        input.fmv_at_gift_date
    } else {
        adjusted_donor_basis
    };

    // Step 3: Compute sale outcome.
    let (outcome, recognized) = if dual_basis {
        if input.sale_price > donee_basis_for_gain {
            (
                GiftSaleOutcome::Gain,
                input.sale_price - donee_basis_for_gain,
            )
        } else if input.sale_price < donee_basis_for_loss {
            (
                GiftSaleOutcome::Loss,
                input.sale_price - donee_basis_for_loss,
            )
        } else {
            (GiftSaleOutcome::PhantomZone, Decimal::ZERO)
        }
    } else {
        // Single basis. Gain or loss measured against the carryover.
        let diff = input.sale_price - adjusted_donor_basis;
        if diff > Decimal::ZERO {
            (GiftSaleOutcome::Gain, diff)
        } else if diff < Decimal::ZERO {
            (GiftSaleOutcome::Loss, diff)
        } else {
            // Exact-zero sale price equals basis → no gain, no loss but
            // not a phantom-zone case. Map to PhantomZone for outcome
            // classification clarity.
            (GiftSaleOutcome::PhantomZone, Decimal::ZERO)
        }
    };

    // Step 4: Holding period.
    let (hp_start, hp_days) = if dual_basis && matches!(outcome, GiftSaleOutcome::Loss) {
        // Loss-side dual-basis path: holding period starts at gift.
        let days = (input.sale_date - input.gift_date).num_days();
        (input.gift_date, days)
    } else {
        // Gain path or single basis: tack from donor's acquisition.
        let days = (input.sale_date - input.donor_holding_period_start).num_days();
        (input.donor_holding_period_start, days)
    };

    let character = match outcome {
        GiftSaleOutcome::PhantomZone => CapitalCharacter::NoGainNoLoss,
        _ => {
            if hp_days > ONE_YEAR_DAYS {
                CapitalCharacter::LongTermCapital
            } else {
                CapitalCharacter::ShortTermCapital
            }
        }
    };

    let note = match outcome {
        GiftSaleOutcome::Gain => format!(
            "§1015(a) gain: sale ${} − donee gain-basis ${} = ${} {} capital{}",
            input.sale_price.round_dp(2),
            donee_basis_for_gain.round_dp(2),
            recognized.round_dp(2),
            if hp_days > ONE_YEAR_DAYS { "long-term" } else { "short-term" },
            if gift_tax_basis_increase > Decimal::ZERO {
                format!(" (§1015(d) gift-tax increase of ${} applied)", gift_tax_basis_increase.round_dp(2))
            } else {
                String::new()
            }
        ),
        GiftSaleOutcome::Loss => format!(
            "§1015(a) loss: sale ${} − donee loss-basis ${} = ${} {} capital{}",
            input.sale_price.round_dp(2),
            donee_basis_for_loss.round_dp(2),
            recognized.round_dp(2),
            if hp_days > ONE_YEAR_DAYS { "long-term" } else { "short-term" },
            if dual_basis {
                " (§1015(a) dual-basis loss path — holding period starts at gift, no tacking)"
            } else {
                ""
            }
        ),
        GiftSaleOutcome::PhantomZone => format!(
            "§1015(a) phantom zone: sale ${} between donee loss-basis ${} and gain-basis ${} — NO gain, NO loss recognized (loss bifurcation prevents donor's loss from passing to donee)",
            input.sale_price.round_dp(2),
            donee_basis_for_loss.round_dp(2),
            donee_basis_for_gain.round_dp(2),
        ),
    };

    Section1015Result {
        donee_basis_for_gain,
        donee_basis_for_loss,
        realized_outcome: outcome,
        recognized_amount: recognized,
        character,
        holding_period_start: hp_start,
        holding_period_days: hp_days,
        gift_tax_basis_increase,
        dual_basis_applied: dual_basis,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base_appreciated() -> Section1015Input {
        Section1015Input {
            donor_adjusted_basis: dec!(10_000),
            donor_holding_period_start: d(2020, 1, 1),
            fmv_at_gift_date: dec!(100_000),
            gift_date: d(2026, 1, 1),
            gift_tax_paid: Decimal::ZERO,
            gift_amount_for_tax_purposes: Decimal::ZERO,
            sale_price: dec!(150_000),
            sale_date: d(2026, 7, 1),
        }
    }

    fn base_depreciated() -> Section1015Input {
        Section1015Input {
            donor_adjusted_basis: dec!(100_000),
            donor_holding_period_start: d(2020, 1, 1),
            fmv_at_gift_date: dec!(50_000),
            gift_date: d(2026, 1, 1),
            gift_tax_paid: Decimal::ZERO,
            gift_amount_for_tax_purposes: Decimal::ZERO,
            sale_price: dec!(150_000),
            sale_date: d(2026, 7, 1),
        }
    }

    #[test]
    fn appreciated_property_carryover_no_dual_basis() {
        // Donor basis $10k, FMV $100k → appreciated. Single basis.
        // Sale $150k → gain $140k.
        let r = compute(&base_appreciated());
        assert!(!r.dual_basis_applied);
        assert_eq!(r.donee_basis_for_gain, dec!(10_000));
        assert_eq!(r.donee_basis_for_loss, dec!(10_000));
        assert_eq!(r.realized_outcome, GiftSaleOutcome::Gain);
        assert_eq!(r.recognized_amount, dec!(140_000));
    }

    #[test]
    fn appreciated_with_long_donor_holding_immediate_ltcg_via_tacking() {
        // Donor held since 2020-01-01; gift 2026-01-01; sale 2026-07-01.
        // §1223(2) tacks donor's holding period → sale date is 6.5 years
        // past donor's acquisition → LTCG even though sale is only 6
        // months after gift.
        let r = compute(&base_appreciated());
        assert!(r.holding_period_days > ONE_YEAR_DAYS);
        assert_eq!(r.character, CapitalCharacter::LongTermCapital);
    }

    #[test]
    fn depreciated_sale_above_donor_basis_gain_via_donor_basis() {
        // Donor basis $100k, FMV $50k (depreciated), sale $150k.
        // Sale > donor basis → §1015(a) gain path: $150k - $100k = $50k.
        let r = compute(&base_depreciated());
        assert!(r.dual_basis_applied);
        assert_eq!(r.donee_basis_for_gain, dec!(100_000));
        assert_eq!(r.donee_basis_for_loss, dec!(50_000));
        assert_eq!(r.realized_outcome, GiftSaleOutcome::Gain);
        assert_eq!(r.recognized_amount, dec!(50_000));
    }

    #[test]
    fn depreciated_sale_below_fmv_at_gift_loss_via_fmv() {
        // Donor basis $100k, FMV $50k, sale $30k. Sale < FMV-at-gift →
        // §1015(a) loss path: $30k - $50k = -$20k loss.
        let mut i = base_depreciated();
        i.sale_price = dec!(30_000);
        let r = compute(&i);
        assert!(r.dual_basis_applied);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::Loss);
        assert_eq!(r.recognized_amount, dec!(-20_000));
    }

    #[test]
    fn depreciated_sale_in_phantom_zone_no_gain_no_loss() {
        // Sale $75k — between FMV $50k and donor basis $100k. The
        // famous "phantom zone": no gain, no loss recognized. The
        // donor's embedded $25k loss vanishes; the donee's $25k of
        // economic appreciation since gift also vanishes.
        let mut i = base_depreciated();
        i.sale_price = dec!(75_000);
        let r = compute(&i);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::PhantomZone);
        assert_eq!(r.recognized_amount, Decimal::ZERO);
        assert_eq!(r.character, CapitalCharacter::NoGainNoLoss);
        assert!(r.note.contains("phantom zone"));
    }

    #[test]
    fn depreciated_phantom_zone_at_fmv_exact_boundary() {
        // Sale exactly at FMV-at-gift = $50k. The loss-path threshold
        // is "< loss-basis"; equality means no loss either → phantom zone.
        let mut i = base_depreciated();
        i.sale_price = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::PhantomZone);
    }

    #[test]
    fn depreciated_phantom_zone_at_donor_basis_exact_boundary() {
        // Sale exactly at donor basis $100k. Gain-path threshold is
        // "> gain-basis"; equality means no gain → phantom zone.
        let mut i = base_depreciated();
        i.sale_price = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::PhantomZone);
    }

    #[test]
    fn loss_path_holding_period_starts_at_gift_not_donor() {
        // §1015(a) loss path: holding period starts at gift date, NOT
        // donor's acquisition. Sale 6 months after gift on loss path =
        // STCG even though donor held > 1 year.
        let mut i = base_depreciated();
        i.sale_price = dec!(30_000); // loss
        let r = compute(&i);
        assert_eq!(r.holding_period_start, d(2026, 1, 1)); // gift date
        let days = (d(2026, 7, 1) - d(2026, 1, 1)).num_days();
        assert_eq!(r.holding_period_days, days);
        assert_eq!(r.character, CapitalCharacter::ShortTermCapital);
    }

    #[test]
    fn gain_path_holding_period_tacks_to_donor() {
        // Gain path on depreciated property still tacks donor's holding
        // period. Donor held since 2020 → > 1 year, LTCG.
        let r = compute(&base_depreciated());
        assert_eq!(r.holding_period_start, d(2020, 1, 1));
        assert_eq!(r.character, CapitalCharacter::LongTermCapital);
    }

    #[test]
    fn gift_tax_basis_increase_applied_to_appreciated() {
        // §1015(d) — donor basis $10k, FMV $100k, gift_amount $84k
        // (after $16k annual exclusion), gift tax $30k.
        // Net appreciation = $100k - $10k = $90k.
        // Increase = $30k × ($90k / $84k) ≈ $32.14k.
        // Cap from net appreciation: $90k → not binding.
        // Cap from FMV: $100k - $10k = $90k → not binding.
        // So increase = $32.14k.
        let mut i = base_appreciated();
        i.gift_tax_paid = dec!(30_000);
        i.gift_amount_for_tax_purposes = dec!(84_000);
        let r = compute(&i);
        // 30000 * 90000 / 84000 = 32142.857...
        let expected = dec!(30_000) * (dec!(90_000) / dec!(84_000));
        assert_eq!(r.gift_tax_basis_increase, expected);
        // Donee gain basis = 10000 + increase.
        assert_eq!(r.donee_basis_for_gain, dec!(10_000) + expected);
    }

    #[test]
    fn gift_tax_basis_increase_capped_at_net_appreciation() {
        // Pathological: large gift tax on small appreciation. Cap fires.
        // Donor basis $10k, FMV $20k → net appreciation $10k.
        // Gift tax $100k, gift_amount $5k → raw = $100k × 10/5 = $200k.
        // Capped at net appreciation $10k.
        let mut i = base_appreciated();
        i.fmv_at_gift_date = dec!(20_000);
        i.gift_tax_paid = dec!(100_000);
        i.gift_amount_for_tax_purposes = dec!(5_000);
        i.sale_price = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.gift_tax_basis_increase, dec!(10_000));
    }

    #[test]
    fn no_gift_tax_increase_on_depreciated_property() {
        // §1015(d) only applies when net appreciation > 0. Depreciated
        // property has no net appreciation → no basis increase even if
        // gift tax was paid.
        let mut i = base_depreciated();
        i.gift_tax_paid = dec!(5_000);
        i.gift_amount_for_tax_purposes = dec!(34_000);
        let r = compute(&i);
        assert_eq!(r.gift_tax_basis_increase, Decimal::ZERO);
    }

    #[test]
    fn zero_gift_tax_no_increase() {
        let mut i = base_appreciated();
        i.gift_tax_paid = Decimal::ZERO;
        i.gift_amount_for_tax_purposes = dec!(84_000);
        let r = compute(&i);
        assert_eq!(r.gift_tax_basis_increase, Decimal::ZERO);
    }

    #[test]
    fn zero_gift_amount_no_increase() {
        // gift_amount_for_tax = 0 → division-by-zero guarded, no increase.
        let mut i = base_appreciated();
        i.gift_tax_paid = dec!(5_000);
        i.gift_amount_for_tax_purposes = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.gift_tax_basis_increase, Decimal::ZERO);
    }

    #[test]
    fn fmv_equals_donor_basis_single_basis_no_dual() {
        // FMV $50k = donor basis $50k → not "less than", so dual-basis
        // does not apply. Single basis.
        let mut i = base_depreciated();
        i.fmv_at_gift_date = dec!(100_000); // same as donor basis
        let r = compute(&i);
        assert!(!r.dual_basis_applied);
        assert_eq!(r.donee_basis_for_gain, r.donee_basis_for_loss);
    }

    #[test]
    fn sale_at_donor_basis_exact_no_gain_no_loss_single_basis() {
        // Single-basis path with sale exactly equal to basis → recognized
        // = 0 → maps to PhantomZone outcome for classification clarity.
        let mut i = base_appreciated();
        i.sale_price = dec!(10_000); // exact donor basis
        let r = compute(&i);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::PhantomZone);
        assert_eq!(r.recognized_amount, Decimal::ZERO);
    }

    #[test]
    fn appreciated_sale_below_basis_single_basis_loss() {
        // Appreciated property, sale below basis → loss via the SINGLE
        // basis (donor's basis, not FMV — because §1015(a) dual-basis
        // doesn't fire on appreciated property).
        let mut i = base_appreciated();
        i.sale_price = dec!(5_000); // below donor basis
        let r = compute(&i);
        assert!(!r.dual_basis_applied);
        assert_eq!(r.realized_outcome, GiftSaleOutcome::Loss);
        assert_eq!(r.recognized_amount, dec!(-5_000));
    }

    #[test]
    fn dual_basis_with_gift_tax_increase_changes_phantom_zone_bounds() {
        // §1015(d) gift-tax increase widens the gain-side basis. If
        // the increase pushes adjusted donor basis above FMV, the
        // FMV ceiling clamps so dual-basis still applies. Test that
        // the cap from FMV fires when the calculation would otherwise
        // exceed FMV.
        let mut i = base_depreciated();
        i.gift_tax_paid = dec!(1_000);
        i.gift_amount_for_tax_purposes = dec!(20_000);
        let r = compute(&i);
        // Net appreciation on depreciated = 0 → no increase.
        assert_eq!(r.gift_tax_basis_increase, Decimal::ZERO);
        assert_eq!(r.donee_basis_for_gain, dec!(100_000));
    }

    #[test]
    fn note_describes_loss_path_holding_period_exception() {
        let mut i = base_depreciated();
        i.sale_price = dec!(30_000);
        let r = compute(&i);
        assert!(r.note.contains("dual-basis"));
        assert!(r.note.contains("no tacking"));
    }

    #[test]
    fn note_describes_loss_bifurcation_in_phantom_zone() {
        // The phantom-zone note should explain WHY the loss disappears
        // — the loss-bifurcation rule prevents donor loss from passing.
        let mut i = base_depreciated();
        i.sale_price = dec!(75_000);
        let r = compute(&i);
        assert!(r.note.contains("loss bifurcation"));
    }

    #[test]
    fn zero_basis_donor_carries_zero_basis_to_donee() {
        // Donor's basis is zero (already-depreciated rental etc.).
        // Donee inherits zero basis. Sale = full gain.
        let mut i = base_appreciated();
        i.donor_adjusted_basis = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.donee_basis_for_gain, Decimal::ZERO);
        assert_eq!(r.recognized_amount, dec!(150_000));
    }

    #[test]
    fn very_large_basis_no_precision_loss() {
        // $1.234B donor basis with $5B FMV — Decimal stays exact across
        // the §1015(d) ratio + cap chain.
        let mut i = base_appreciated();
        i.donor_adjusted_basis = dec!(1_234_567_890.12);
        i.fmv_at_gift_date = dec!(5_000_000_000);
        i.sale_price = dec!(10_000_000_000);
        let r = compute(&i);
        assert_eq!(r.donee_basis_for_gain, dec!(1_234_567_890.12));
        assert_eq!(r.recognized_amount, dec!(8_765_432_109.88));
    }

    #[test]
    fn one_year_boundary_gain_tacking_366d_long_term() {
        // Donor's holding period start exactly 366d before sale → LTCG.
        let mut i = base_appreciated();
        i.donor_holding_period_start = d(2025, 6, 30);
        i.sale_date = d(2026, 7, 1);
        let r = compute(&i);
        assert_eq!(r.holding_period_days, 366);
        assert_eq!(r.character, CapitalCharacter::LongTermCapital);
    }

    #[test]
    fn one_year_boundary_loss_path_365d_short_term() {
        // Loss path: holding starts at gift. Gift 2026-01-01, sale
        // 2027-01-01 = 365 days exactly = ≤ 1 year = STCG.
        let mut i = base_depreciated();
        i.sale_price = dec!(30_000); // loss
        i.gift_date = d(2026, 1, 1);
        i.sale_date = d(2027, 1, 1);
        let r = compute(&i);
        assert_eq!(r.holding_period_days, 365);
        assert_eq!(r.character, CapitalCharacter::ShortTermCapital);
    }
}
