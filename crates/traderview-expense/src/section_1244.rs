//! IRC §1244 — Small Business Stock Loss.
//!
//! Most traders treat all stock losses as capital losses, hitting the
//! §1212(b) $3,000 / $1,500-MFS annual cap against ordinary income.
//! §1244 lets you **bypass that cap** for losses on qualifying small-
//! business stock: the first **$50,000 single / $100,000 MFJ** of
//! such loss in any tax year is treated as **ordinary loss**, not
//! capital loss. Anything above the cap reverts to capital loss
//! treatment and stacks back onto Schedule D / §1212(b).
//!
//! Qualification under §1244(c) requires ALL of:
//!
//!   * Stock issued by a **domestic** C-corp or S-corp.
//!   * Aggregate **paid-in capital + paid-in surplus ≤ $1,000,000**
//!     at issuance (small-business cap).
//!   * For the 5 years preceding the year of loss, the corp's gross
//!     receipts must be **less than 50%** from royalties, rents,
//!     dividends, interest, annuities, or sales/exchanges of stock
//!     or securities (the "non-passive operating business" test).
//!   * Stock issued **for money or other property** (not for services,
//!     not for other stock).
//!   * Taxpayer is the **original holder** (no inherited / gifted /
//!     purchased-on-secondary-market stock qualifies).
//!
//! The corp side qualifies the *stock*; the §1244 ordinary-loss
//! treatment is a *taxpayer-level* election that hits the $50k/$100k
//! cap per year, not per stock or per corp.
//!
//! Pure compute. Caller asserts qualification (we expose a `Qualified`
//! struct with each boolean test for transparency); we compute the
//! split between ordinary and capital portions.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

/// §1244(c) qualification checklist. Each field is a yes/no — `true`
/// means the test passes. A loss only gets §1244 treatment when ALL
/// five tests pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qualification {
    /// Stock issued by a domestic C-corp or S-corp.
    pub domestic_corporation: bool,
    /// Aggregate paid-in capital + paid-in surplus ≤ $1M at issuance.
    pub paid_in_capital_under_1m_at_issuance: bool,
    /// For the 5 years before the loss, < 50% of gross receipts came
    /// from passive sources (royalties / rents / dividends / interest
    /// / annuities / sales of stock or securities).
    pub gross_receipts_non_passive_5y: bool,
    /// Stock issued for money or other property (not services).
    pub issued_for_money_or_property: bool,
    /// Taxpayer is the original holder (not inherited / gifted /
    /// purchased on secondary market).
    pub original_holder: bool,
}

impl Qualification {
    pub fn qualifies(&self) -> bool {
        self.domestic_corporation
            && self.paid_in_capital_under_1m_at_issuance
            && self.gross_receipts_non_passive_5y
            && self.issued_for_money_or_property
            && self.original_holder
    }

    /// Human-readable list of failures — used in the result note so
    /// the user knows WHY a position didn't qualify.
    fn failures(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if !self.domestic_corporation {
            v.push("not a domestic corporation");
        }
        if !self.paid_in_capital_under_1m_at_issuance {
            v.push("aggregate paid-in capital > $1M at issuance");
        }
        if !self.gross_receipts_non_passive_5y {
            v.push("≥50% gross receipts from passive sources (5-year test)");
        }
        if !self.issued_for_money_or_property {
            v.push("stock issued for services, not money/property");
        }
        if !self.original_holder {
            v.push("taxpayer is not the original holder");
        }
        v
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1244Input {
    /// Total loss realized on the qualifying small-business stock
    /// (positive number — we're computing a loss, sign-flip on the way out).
    pub realized_loss: Decimal,
    pub filing_status: FilingStatus,
    /// Sum of §1244 ordinary-loss treatment ALREADY claimed earlier
    /// this tax year on OTHER §1244 dispositions. Lets multi-stock
    /// dispositions stack against the same $50k / $100k cap.
    pub ordinary_loss_claimed_this_year_so_far: Decimal,
    pub qualification: Qualification,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1244Result {
    /// §1244 ordinary-loss deduction for THIS disposition (after
    /// applying the cap and prior-claimed portion this year).
    pub ordinary_loss_deduction: Decimal,
    /// Portion that overflows to capital loss treatment (Schedule D).
    pub capital_loss_overflow: Decimal,
    /// The $50k / $100k cap applicable to this filing status.
    pub annual_cap: Decimal,
    /// Cap remaining for this tax year AFTER this disposition.
    pub cap_remaining_after: Decimal,
    /// True when the qualification checklist did NOT pass.
    pub disqualified: bool,
    pub note: String,
}

fn annual_cap(fs: FilingStatus) -> Decimal {
    match fs {
        // §1244(b): $100k for joint return; $50k for any other return.
        FilingStatus::MarriedFilingJointly => Decimal::from_str("100000").unwrap(),
        _ => Decimal::from_str("50000").unwrap(),
    }
}

pub fn compute(input: &Section1244Input) -> Section1244Result {
    let mut r = Section1244Result {
        annual_cap: annual_cap(input.filing_status),
        ..Section1244Result::default()
    };

    if input.realized_loss <= Decimal::ZERO {
        r.note = "no loss to apply §1244 against".into();
        r.cap_remaining_after = r.annual_cap - input.ordinary_loss_claimed_this_year_so_far;
        return r;
    }

    if !input.qualification.qualifies() {
        r.disqualified = true;
        r.capital_loss_overflow = input.realized_loss; // full loss flows to Schedule D
        r.cap_remaining_after = r.annual_cap - input.ordinary_loss_claimed_this_year_so_far;
        let reasons = input.qualification.failures().join(", ");
        r.note = format!("§1244 disqualified ({reasons}); full loss treated as capital loss");
        return r;
    }

    let cap_remaining_before =
        (r.annual_cap - input.ordinary_loss_claimed_this_year_so_far).max(Decimal::ZERO);
    r.ordinary_loss_deduction = input.realized_loss.min(cap_remaining_before);
    r.capital_loss_overflow = input.realized_loss - r.ordinary_loss_deduction;
    r.cap_remaining_after = cap_remaining_before - r.ordinary_loss_deduction;

    r.note = if r.capital_loss_overflow > Decimal::ZERO {
        format!(
            "§1244 ordinary loss ${} (cap hit); ${} overflow to Schedule D",
            r.ordinary_loss_deduction, r.capital_loss_overflow
        )
    } else {
        format!(
            "§1244 ordinary loss ${}; ${} cap remaining for the year",
            r.ordinary_loss_deduction, r.cap_remaining_after
        )
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn qualified() -> Qualification {
        Qualification {
            domestic_corporation: true,
            paid_in_capital_under_1m_at_issuance: true,
            gross_receipts_non_passive_5y: true,
            issued_for_money_or_property: true,
            original_holder: true,
        }
    }

    fn base(loss: Decimal, fs: FilingStatus) -> Section1244Input {
        Section1244Input {
            realized_loss: loss,
            filing_status: fs,
            ordinary_loss_claimed_this_year_so_far: Decimal::ZERO,
            qualification: qualified(),
        }
    }

    #[test]
    fn single_full_50k_ordinary_below_cap() {
        // $30k loss, single → full $30k ordinary, no overflow.
        let r = compute(&base(dec!(30000), FilingStatus::Single));
        assert_eq!(r.annual_cap, dec!(50000));
        assert_eq!(r.ordinary_loss_deduction, dec!(30000));
        assert_eq!(r.capital_loss_overflow, Decimal::ZERO);
        assert_eq!(r.cap_remaining_after, dec!(20000));
    }

    #[test]
    fn single_loss_above_cap_overflows_to_capital_loss() {
        // $70k loss, single → $50k ordinary, $20k capital overflow.
        let r = compute(&base(dec!(70000), FilingStatus::Single));
        assert_eq!(r.ordinary_loss_deduction, dec!(50000));
        assert_eq!(r.capital_loss_overflow, dec!(20000));
        assert_eq!(r.cap_remaining_after, Decimal::ZERO);
    }

    #[test]
    fn mfj_cap_is_100k() {
        let r = compute(&base(dec!(80000), FilingStatus::MarriedFilingJointly));
        assert_eq!(r.annual_cap, dec!(100000));
        assert_eq!(r.ordinary_loss_deduction, dec!(80000));
        assert_eq!(r.capital_loss_overflow, Decimal::ZERO);
    }

    #[test]
    fn mfs_uses_single_50k_cap_not_50_pct_of_mfj() {
        // §1244(b): explicit 50k for "any other return" including MFS.
        let r = compute(&base(dec!(70000), FilingStatus::MarriedFilingSeparately));
        assert_eq!(r.annual_cap, dec!(50000));
        assert_eq!(r.ordinary_loss_deduction, dec!(50000));
        assert_eq!(r.capital_loss_overflow, dec!(20000));
    }

    #[test]
    fn prior_claimed_reduces_remaining_cap() {
        // Already claimed $40k this year, $30k new loss, single.
        // Cap remaining = $50k - $40k = $10k → ordinary $10k, overflow $20k.
        let mut i = base(dec!(30000), FilingStatus::Single);
        i.ordinary_loss_claimed_this_year_so_far = dec!(40000);
        let r = compute(&i);
        assert_eq!(r.ordinary_loss_deduction, dec!(10000));
        assert_eq!(r.capital_loss_overflow, dec!(20000));
        assert_eq!(r.cap_remaining_after, Decimal::ZERO);
    }

    #[test]
    fn disqualification_routes_full_loss_to_capital() {
        let mut i = base(dec!(40000), FilingStatus::Single);
        i.qualification.original_holder = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert_eq!(r.ordinary_loss_deduction, Decimal::ZERO);
        assert_eq!(r.capital_loss_overflow, dec!(40000));
        assert!(r.note.contains("original holder"));
    }

    #[test]
    fn multiple_disqualifications_listed_in_note() {
        let mut i = base(dec!(40000), FilingStatus::Single);
        i.qualification.domestic_corporation = false;
        i.qualification.original_holder = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("domestic corporation"));
        assert!(r.note.contains("original holder"));
    }

    #[test]
    fn no_loss_no_op() {
        let r = compute(&base(Decimal::ZERO, FilingStatus::Single));
        assert_eq!(r.ordinary_loss_deduction, Decimal::ZERO);
        assert!(r.note.contains("no loss"));
    }

    #[test]
    fn cap_remaining_never_negative_under_stress() {
        let mut i = base(dec!(1000000), FilingStatus::Single);
        i.ordinary_loss_claimed_this_year_so_far = dec!(60000); // already over cap
        let r = compute(&i);
        assert_eq!(r.ordinary_loss_deduction, Decimal::ZERO);
        assert_eq!(r.capital_loss_overflow, dec!(1000000));
        assert_eq!(r.cap_remaining_after, Decimal::ZERO);
    }

    #[test]
    fn paid_in_over_1m_disqualifies() {
        let mut i = base(dec!(40000), FilingStatus::Single);
        i.qualification.paid_in_capital_under_1m_at_issuance = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("paid-in capital"));
    }

    #[test]
    fn five_year_passive_gross_receipts_disqualifies() {
        let mut i = base(dec!(40000), FilingStatus::Single);
        i.qualification.gross_receipts_non_passive_5y = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("passive sources"));
    }

    #[test]
    fn issued_for_services_disqualifies() {
        let mut i = base(dec!(40000), FilingStatus::Single);
        i.qualification.issued_for_money_or_property = false;
        let r = compute(&i);
        assert!(r.disqualified);
        assert!(r.note.contains("services"));
    }

    #[test]
    fn exact_cap_loss_no_overflow() {
        // $50k loss single → exactly fills cap, zero overflow, zero remaining.
        let r = compute(&base(dec!(50000), FilingStatus::Single));
        assert_eq!(r.ordinary_loss_deduction, dec!(50000));
        assert_eq!(r.capital_loss_overflow, Decimal::ZERO);
        assert_eq!(r.cap_remaining_after, Decimal::ZERO);
    }

    #[test]
    fn qualification_helper_returns_true_only_when_all_five_pass() {
        let mut q = qualified();
        assert!(q.qualifies());
        q.domestic_corporation = false;
        assert!(!q.qualifies());
        q.domestic_corporation = true;
        q.original_holder = false;
        assert!(!q.qualifies());
    }
}
