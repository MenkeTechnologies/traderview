//! IRC §1045 — Rollover of gain from qualified small business stock.
//!
//! §1202 (see `section_1202`) excludes gain only after the original
//! QSBS is held **5 years**. §1045 plugs the gap before that
//! milestone: a taxpayer who has held QSBS for **more than 6 months**
//! and sells before reaching 5 years can **DEFER** the gain by
//! reinvesting the proceeds into OTHER QSBS within **60 days** of
//! the sale.
//!
//! The replacement QSBS picks up a carryover basis (replacement cost
//! minus deferred gain), and the original's holding period **tacks
//! onto** the replacement for the §1202 5-year clock. Chaining sales
//! through multiple rollovers eventually qualifies for full §1202
//! exclusion at zero cost in deferred basis.
//!
//! Mechanics:
//!
//!   * Gain deferred = MIN(realized gain, replacement cost). Any
//!     cash kept ("boot") triggers recognition up to the realized gain.
//!   * Boot received = sale_proceeds_net - replacement_cost
//!     (when replacement_cost < sale_proceeds_net).
//!   * Replacement basis = replacement_cost - gain_deferred. This is
//!     the carryover basis that preserves the deferred gain for later.
//!   * Holding-period tack-on: replacement gets the original's
//!     holding period for §1202 purposes (the 5-year clock is
//!     continuous across rollovers).
//!
//! Qualification:
//!
//!   * Original stock satisfies §1202(c) qualification (caller asserts
//!     via the same 8-bool checklist used in `section_1202`).
//!   * Original held > 6 months.
//!   * Replacement purchased within 60 days of original sale.
//!   * Replacement stock independently qualifies under §1202(c).
//!
//! Pure compute. Caller asserts qualifications and supplies dates +
//! dollar amounts; we compute deferral, recognized boot, and the
//! replacement basis + new holding-period-start date.

use chrono::{Duration, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1045Input {
    pub realized_gain: Decimal,
    /// Cash proceeds NET of selling costs.
    pub sale_proceeds_net: Decimal,
    /// Cost basis the taxpayer rolls into the replacement QSBS.
    pub replacement_cost: Decimal,
    pub original_acquisition_date: NaiveDate,
    pub original_sale_date: NaiveDate,
    pub replacement_purchase_date: NaiveDate,
    /// True if original stock met §1202(c) qualification.
    pub original_qsbs_qualified: bool,
    /// True if replacement stock independently meets §1202(c).
    pub replacement_qsbs_qualified: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1045Result {
    pub days_held_original: i64,
    pub days_to_replacement: i64,
    pub gain_deferred: Decimal,
    /// Boot recognized: realized - deferred. Taxed this year as gain.
    pub gain_recognized_this_year: Decimal,
    /// Replacement basis = replacement_cost - gain_deferred.
    pub replacement_basis: Decimal,
    /// Effective holding-period-start date for §1202 5-year clock —
    /// inherits the original's acquisition date when §1045 applies.
    pub effective_holding_period_start: Option<NaiveDate>,
    pub disqualified: bool,
    pub disqualification_reasons: Vec<String>,
    pub note: String,
}

pub fn compute(input: &Section1045Input) -> Section1045Result {
    let mut r = Section1045Result {
        days_held_original: (input.original_sale_date - input.original_acquisition_date)
            .num_days(),
        days_to_replacement: (input.replacement_purchase_date - input.original_sale_date)
            .num_days(),
        ..Section1045Result::default()
    };

    let six_months = Duration::days(183);
    let sixty_days = Duration::days(60);

    if !input.original_qsbs_qualified {
        r.disqualification_reasons
            .push("original stock not §1202-qualified".into());
    }
    if !input.replacement_qsbs_qualified {
        r.disqualification_reasons
            .push("replacement stock not §1202-qualified".into());
    }
    if (input.original_sale_date - input.original_acquisition_date) <= six_months {
        r.disqualification_reasons.push(format!(
            "original held only {} days ≤ 6 months (>183 required)",
            r.days_held_original
        ));
    }
    let replacement_lag = input.replacement_purchase_date - input.original_sale_date;
    if replacement_lag < Duration::days(0) {
        r.disqualification_reasons
            .push("replacement purchased BEFORE original sale".into());
    } else if replacement_lag > sixty_days {
        r.disqualification_reasons.push(format!(
            "replacement bought {} days after sale > 60-day §1045 window",
            r.days_to_replacement
        ));
    }
    if input.realized_gain <= Decimal::ZERO {
        r.note = "no gain to defer (§1045 applies only to gains)".into();
        return r;
    }

    if !r.disqualification_reasons.is_empty() {
        r.disqualified = true;
        r.gain_recognized_this_year = input.realized_gain;
        r.note = format!(
            "§1045 disqualified: {}",
            r.disqualification_reasons.join("; ")
        );
        return r;
    }

    // Deferral mechanics. If replacement cost >= sale proceeds, full
    // gain is deferred. Otherwise the shortfall is boot, recognized
    // up to realized gain.
    let boot = (input.sale_proceeds_net - input.replacement_cost).max(Decimal::ZERO);
    r.gain_recognized_this_year = input.realized_gain.min(boot).max(Decimal::ZERO);
    r.gain_deferred = input.realized_gain - r.gain_recognized_this_year;
    r.replacement_basis = (input.replacement_cost - r.gain_deferred).max(Decimal::ZERO);
    // §1045(b)(4) holding-period tack: replacement inherits the
    // original's acquisition date for §1202's 5-year clock.
    r.effective_holding_period_start = Some(input.original_acquisition_date);

    r.note = if r.gain_recognized_this_year > Decimal::ZERO {
        format!(
            "§1045: ${} deferred, ${} boot recognized; replacement basis ${} carries forward",
            r.gain_deferred, r.gain_recognized_this_year, r.replacement_basis,
        )
    } else {
        format!(
            "§1045: full ${} deferred; replacement basis ${} carries forward, holding period tacks from {}",
            r.gain_deferred,
            r.replacement_basis,
            input.original_acquisition_date,
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn base() -> Section1045Input {
        // Original acquired Jan 2022, sold Jan 2023 ($1M gain). Held 1 year.
        // Rolled into replacement same month for $5M.
        Section1045Input {
            realized_gain: dec!(1000000),
            sale_proceeds_net: dec!(5000000),
            replacement_cost: dec!(5000000),
            original_acquisition_date: d(2022, 1, 1),
            original_sale_date: d(2023, 1, 15),
            replacement_purchase_date: d(2023, 1, 20),
            original_qsbs_qualified: true,
            replacement_qsbs_qualified: true,
        }
    }

    #[test]
    fn full_replacement_no_boot_full_deferral() {
        let r = compute(&base());
        assert_eq!(r.gain_deferred, dec!(1000000));
        assert_eq!(r.gain_recognized_this_year, Decimal::ZERO);
        // Replacement basis = $5M - $1M deferred = $4M.
        assert_eq!(r.replacement_basis, dec!(4000000));
        assert_eq!(r.effective_holding_period_start, Some(d(2022, 1, 1)));
        assert!(!r.disqualified);
    }

    #[test]
    fn partial_replacement_boot_recognized() {
        // Replacement $4.5M, sale proceeds $5M → $500k boot.
        // Recognized = MIN($1M gain, $500k boot) = $500k.
        // Deferred = $500k. Replacement basis = $4.5M - $500k = $4M.
        let mut i = base();
        i.replacement_cost = dec!(4500000);
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, dec!(500000));
        assert_eq!(r.gain_deferred, dec!(500000));
        assert_eq!(r.replacement_basis, dec!(4000000));
    }

    #[test]
    fn boot_exceeds_gain_caps_recognition_at_gain() {
        // Replacement only $1M, sale proceeds $5M → $4M boot.
        // Recognized = MIN($1M gain, $4M boot) = $1M.
        // Deferred = $0. Replacement basis = $1M - $0 = $1M.
        let mut i = base();
        i.replacement_cost = dec!(1000000);
        let r = compute(&i);
        assert_eq!(r.gain_recognized_this_year, dec!(1000000));
        assert_eq!(r.gain_deferred, Decimal::ZERO);
        assert_eq!(r.replacement_basis, dec!(1000000));
    }

    #[test]
    fn held_under_6_months_disqualified() {
        let mut i = base();
        i.original_sale_date = d(2022, 6, 1); // 5 months hold
        i.replacement_purchase_date = d(2022, 6, 15);
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("6 months")));
        assert_eq!(r.gain_recognized_this_year, dec!(1000000));
    }

    #[test]
    fn replacement_after_60_day_window_disqualified() {
        let mut i = base();
        i.replacement_purchase_date = d(2023, 4, 1); // 76 days after sale
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("60-day")));
    }

    #[test]
    fn replacement_before_sale_disqualified() {
        let mut i = base();
        i.replacement_purchase_date = d(2022, 12, 1); // BEFORE sale
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("BEFORE")));
    }

    #[test]
    fn original_not_qsbs_qualified_disqualified() {
        let mut i = base();
        i.original_qsbs_qualified = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("original")));
    }

    #[test]
    fn replacement_not_qsbs_qualified_disqualified() {
        let mut i = base();
        i.replacement_qsbs_qualified = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.iter().any(|s| s.contains("replacement")));
    }

    #[test]
    fn boundary_exactly_6_months_disqualified() {
        // §1045 requires > 6 months. 183 days = boundary; we want strict >.
        let mut i = base();
        i.original_acquisition_date = d(2022, 7, 1);
        i.original_sale_date = d(2022, 12, 31); // 183 days
        i.replacement_purchase_date = d(2023, 1, 5);
        let r = compute(&i);
        assert!(r.disqualified);
    }

    #[test]
    fn boundary_just_over_6_months_qualifies() {
        // 184 days satisfies > 6 months.
        let mut i = base();
        i.original_acquisition_date = d(2022, 7, 1);
        i.original_sale_date = d(2023, 1, 1); // 184 days
        i.replacement_purchase_date = d(2023, 1, 5);
        let r = compute(&i);
        assert!(!r.disqualified);
        assert_eq!(r.gain_deferred, dec!(1000000));
    }

    #[test]
    fn boundary_exactly_60_days_qualifies() {
        let mut i = base();
        i.replacement_purchase_date = i.original_sale_date + Duration::days(60);
        let r = compute(&i);
        assert!(!r.disqualified);
    }

    #[test]
    fn boundary_61_days_disqualified() {
        let mut i = base();
        i.replacement_purchase_date = i.original_sale_date + Duration::days(61);
        let r = compute(&i);
        assert!(r.disqualified);
    }

    #[test]
    fn loss_returns_no_op() {
        let mut i = base();
        i.realized_gain = dec!(-100000);
        let r = compute(&i);
        assert!(r.note.contains("no gain"));
        assert_eq!(r.gain_recognized_this_year, Decimal::ZERO);
    }

    #[test]
    fn holding_period_tacks_to_original_acquisition() {
        // The whole point: rollovers chain the holding period so the
        // §1202 5-year clock is continuous.
        let r = compute(&base());
        assert_eq!(r.effective_holding_period_start, Some(d(2022, 1, 1)));
    }

    #[test]
    fn replacement_basis_never_negative_under_stress() {
        let mut i = base();
        i.realized_gain = dec!(50000000); // bigger than replacement
        i.sale_proceeds_net = dec!(50000000);
        i.replacement_cost = dec!(1000000); // tiny replacement
        let r = compute(&i);
        assert!(r.replacement_basis >= Decimal::ZERO);
    }

    #[test]
    fn full_deferral_zero_basis_carryover_yields_clean_replacement_basis() {
        // Realized $1M, sale $5M, replacement $5M → $4M basis.
        // Verify: replacement_value - replacement_basis = deferred_gain.
        let r = compute(&base());
        let invariant = dec!(5000000) - r.replacement_basis;
        assert_eq!(invariant, r.gain_deferred);
    }

    #[test]
    fn multi_disqualification_lists_all_reasons() {
        let mut i = base();
        i.original_qsbs_qualified = false;
        i.replacement_purchase_date = d(2023, 5, 1); // late
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.disqualification_reasons.len() >= 2);
    }
}
