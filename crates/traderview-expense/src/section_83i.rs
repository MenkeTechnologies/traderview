//! IRC §83(i) — Qualified equity grant 5-year income-tax deferral
//! (TCJA addition).
//!
//! Allows an eligible employee of an eligible private corporation to
//! defer FEDERAL INCOME TAX (only — NOT FICA) on income from
//! non-qualified stock option exercise or RSU vesting for up to 5
//! years. Companion to `section_83b`: where §83(b) accelerates
//! income to grant date to lock in lower FMV, §83(i) defers income
//! away from vest date to wait out the private→public transition.
//!
//! **Eligible private corporation** (§83(i)(2)(C)):
//! 1. No stock of the corp (or any predecessor) is readily tradable
//!    on an established securities market in any prior year.
//! 2. A written plan grants stock options or RSUs to ≥ 80% of all US
//!    employees, with the SAME RIGHTS AND PRIVILEGES to receive
//!    qualified stock.
//!
//! **Excluded employees** (§83(i)(3)(B)) — cannot make the election
//! regardless of corporation status:
//! - 1% owner in the current or any of the **10 preceding** calendar
//!   years (family attribution rules apply)
//! - CEO or CFO at any time during the current or any of the **10
//!   preceding** taxable years (family attribution rules apply)
//! - One of the **4 highest compensated officers** for the current or
//!   any of the 10 preceding years
//!
//! **Deferral period** (§83(i)(1)(B)): ends on the EARLIEST of:
//! - 5 years after the option exercise / RSU settlement
//! - The date the stock becomes transferable to the EMPLOYER (i.e.,
//!   the date a buyback puts cash in the employee's hand)
//! - The date the stock becomes readily tradable on an established
//!   securities market (IPO trigger)
//! - The date the employee becomes an excluded employee
//! - The date the employee revokes the election
//!
//! **FICA NOT deferred**: §83(i) defers only federal income tax.
//! Social Security and Medicare taxes are due at the normal §83(a)
//! vesting/exercise date and the employer must withhold them then.
//! Sources: IRS Notice 2018-97; Trucker Huss 2018 analysis; Tax
//! Adviser November 2021.
//!
//! **30-day election window** (§83(i)(4)(A)): employee must file the
//! deferral election within 30 days after the substantial vesting /
//! exercise date. Missed window = no §83(i) deferral available.

use chrono::{Months, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferralEndTrigger {
    /// 5-year statutory maximum reached.
    FiveYearStatutoryMaximum,
    /// IPO / acquisition — stock now tradable on an established market.
    StockBecameTradable,
    /// Employee revoked the election.
    EmployeeRevoked,
    /// Stock transferred back to employer (buyback / cash-out).
    StockTransferredToEmployer,
    /// Employee became excluded mid-deferral (e.g., promoted to CFO).
    EmployeeBecameExcluded,
    /// Deferral still active as of the as-of date.
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83iInput {
    pub vesting_or_exercise_date: NaiveDate,
    pub deferral_election_date: NaiveDate,
    pub as_of_date: NaiveDate,
    /// True if the corporation meets §83(i)(2)(C) eligible-private-corp
    /// rules: no readily tradable stock + 80% broad-based written plan.
    pub corporation_is_eligible_private: bool,
    pub plan_meets_80_pct_broad_based: bool,
    pub stock_readily_tradable_before_grant: bool,
    /// Employee exclusion flags. ANY true → not eligible for §83(i).
    pub employee_is_1pct_owner_current_or_10y: bool,
    pub employee_is_or_was_ceo_cfo_current_or_10y: bool,
    pub employee_in_top_4_paid_current_or_10y: bool,
    pub deferred_income_amount: Decimal,
    /// FMV-at-vesting basis used to compute FICA, due immediately even
    /// when §83(i) defers income tax.
    pub fmv_at_vesting_for_fica: Decimal,
    /// Effective FICA rate as basis points (SS 6.2% × 2 + Medicare
    /// 1.45% × 2 = 15.3% = 1530 for combined employer+employee;
    /// employee-only side is 7.65% = 765).
    pub fica_combined_rate_bp: u32,
    /// Optional early-trigger dates. Module picks the earliest.
    pub stock_became_tradable_date: Option<NaiveDate>,
    pub employee_revoked_date: Option<NaiveDate>,
    pub stock_transferred_to_employer_date: Option<NaiveDate>,
    pub employee_became_excluded_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section83iResult {
    pub eligible_for_83i_deferral: bool,
    pub election_filed_within_30_days: bool,
    pub five_year_statutory_end_date: NaiveDate,
    pub effective_deferral_end_date: NaiveDate,
    pub trigger: DeferralEndTrigger,
    pub fica_due_at_vesting: Decimal,
    pub income_tax_currently_deferred: Decimal,
    pub income_tax_recognition_year: Option<i32>,
    pub note: String,
}

pub fn compute(input: &Section83iInput) -> Section83iResult {
    let any_exclusion = input.employee_is_1pct_owner_current_or_10y
        || input.employee_is_or_was_ceo_cfo_current_or_10y
        || input.employee_in_top_4_paid_current_or_10y;
    let corp_ok = input.corporation_is_eligible_private
        && input.plan_meets_80_pct_broad_based
        && !input.stock_readily_tradable_before_grant;
    let eligible = corp_ok && !any_exclusion;

    let election_window_end = input
        .vesting_or_exercise_date
        .checked_add_signed(chrono::Duration::days(30))
        .unwrap_or(input.vesting_or_exercise_date);
    let timely = input.deferral_election_date >= input.vesting_or_exercise_date
        && input.deferral_election_date <= election_window_end;

    // 5-year statutory maximum from vest/exercise.
    let five_year_end = input
        .vesting_or_exercise_date
        .checked_add_months(Months::new(60))
        .unwrap_or(input.vesting_or_exercise_date);

    // Earliest trigger wins.
    let mut candidates: Vec<(NaiveDate, DeferralEndTrigger)> =
        vec![(five_year_end, DeferralEndTrigger::FiveYearStatutoryMaximum)];
    if let Some(d) = input.stock_became_tradable_date {
        candidates.push((d, DeferralEndTrigger::StockBecameTradable));
    }
    if let Some(d) = input.employee_revoked_date {
        candidates.push((d, DeferralEndTrigger::EmployeeRevoked));
    }
    if let Some(d) = input.stock_transferred_to_employer_date {
        candidates.push((d, DeferralEndTrigger::StockTransferredToEmployer));
    }
    if let Some(d) = input.employee_became_excluded_date {
        candidates.push((d, DeferralEndTrigger::EmployeeBecameExcluded));
    }
    candidates.sort_by_key(|(d, _)| *d);
    let (end_date, trigger) = candidates[0];

    let active = eligible && timely && input.as_of_date < end_date;
    let final_trigger = if active {
        DeferralEndTrigger::Active
    } else {
        trigger
    };

    // FICA always due at vesting, regardless of §83(i) deferral.
    let fica = input.fmv_at_vesting_for_fica * Decimal::from(input.fica_combined_rate_bp)
        / Decimal::from(10_000);

    let (income_tax_deferred, recognition_year) = if eligible && timely {
        let yr = end_date.format("%Y").to_string().parse::<i32>().ok();
        (input.deferred_income_amount, yr)
    } else {
        (Decimal::ZERO, None)
    };

    let note = if !corp_ok {
        format!(
            "Corporation not §83(i)(2)(C) eligible: private={} broad-based-80%={} not-tradable={}. No deferral available; income tax due at vesting per §83(a).",
            input.corporation_is_eligible_private,
            input.plan_meets_80_pct_broad_based,
            !input.stock_readily_tradable_before_grant,
        )
    } else if any_exclusion {
        format!(
            "Employee is §83(i)(3)(B) excluded: 1%-owner={} CEO/CFO={} top-4-paid={}. No deferral. Family attribution applies. Income tax due per §83(a).",
            input.employee_is_1pct_owner_current_or_10y,
            input.employee_is_or_was_ceo_cfo_current_or_10y,
            input.employee_in_top_4_paid_current_or_10y,
        )
    } else if !timely {
        "Deferral election not made within 30-day §83(i)(4)(A) window from vesting/exercise date. Election void; income tax due per §83(a). FICA still owed."
            .to_string()
    } else {
        format!(
            "§83(i) deferral ELIGIBLE; trigger={:?} ends {}; ${} income tax deferred; ${} FICA (rate {}.{}%) still due NOW at vesting per §3121.",
            final_trigger,
            end_date,
            income_tax_deferred.round_dp(2),
            fica.round_dp(2),
            input.fica_combined_rate_bp / 100,
            input.fica_combined_rate_bp % 100,
        )
    };

    Section83iResult {
        eligible_for_83i_deferral: eligible,
        election_filed_within_30_days: timely,
        five_year_statutory_end_date: five_year_end,
        effective_deferral_end_date: end_date,
        trigger: final_trigger,
        fica_due_at_vesting: fica,
        income_tax_currently_deferred: income_tax_deferred,
        income_tax_recognition_year: recognition_year,
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

    fn base() -> Section83iInput {
        Section83iInput {
            vesting_or_exercise_date: d(2025, 1, 15),
            deferral_election_date: d(2025, 1, 20),
            as_of_date: d(2025, 6, 1),
            corporation_is_eligible_private: true,
            plan_meets_80_pct_broad_based: true,
            stock_readily_tradable_before_grant: false,
            employee_is_1pct_owner_current_or_10y: false,
            employee_is_or_was_ceo_cfo_current_or_10y: false,
            employee_in_top_4_paid_current_or_10y: false,
            deferred_income_amount: dec!(500_000),
            fmv_at_vesting_for_fica: dec!(500_000),
            fica_combined_rate_bp: 1530, // 15.3%
            stock_became_tradable_date: None,
            employee_revoked_date: None,
            stock_transferred_to_employer_date: None,
            employee_became_excluded_date: None,
        }
    }

    #[test]
    fn eligible_baseline_deferral_active_5y_max() {
        let r = compute(&base());
        assert!(r.eligible_for_83i_deferral);
        assert!(r.election_filed_within_30_days);
        assert_eq!(r.effective_deferral_end_date, d(2030, 1, 15));
        assert_eq!(r.trigger, DeferralEndTrigger::Active);
        assert_eq!(r.income_tax_currently_deferred, dec!(500_000));
    }

    #[test]
    fn fica_due_immediately_despite_deferral() {
        // $500k FMV × 15.3% = $76,500 FICA due at vesting.
        let r = compute(&base());
        assert_eq!(r.fica_due_at_vesting, dec!(76_500));
    }

    #[test]
    fn one_percent_owner_excluded_no_deferral() {
        let mut i = base();
        i.employee_is_1pct_owner_current_or_10y = true;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
        assert_eq!(r.income_tax_currently_deferred, Decimal::ZERO);
    }

    #[test]
    fn ceo_cfo_excluded_no_deferral() {
        let mut i = base();
        i.employee_is_or_was_ceo_cfo_current_or_10y = true;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
    }

    #[test]
    fn top_4_paid_excluded_no_deferral() {
        let mut i = base();
        i.employee_in_top_4_paid_current_or_10y = true;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
    }

    #[test]
    fn corp_not_private_no_deferral() {
        let mut i = base();
        i.corporation_is_eligible_private = false;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
        assert!(r.note.contains("not §83(i)(2)(C) eligible"));
    }

    #[test]
    fn plan_fails_80_pct_no_deferral() {
        let mut i = base();
        i.plan_meets_80_pct_broad_based = false;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
        assert!(r.note.contains("broad-based-80%=false"));
    }

    #[test]
    fn stock_already_tradable_disqualifies_corp() {
        let mut i = base();
        i.stock_readily_tradable_before_grant = true;
        let r = compute(&i);
        assert!(!r.eligible_for_83i_deferral);
    }

    #[test]
    fn election_window_day_30_exactly_complies() {
        let mut i = base();
        i.vesting_or_exercise_date = d(2025, 1, 1);
        i.deferral_election_date = d(2025, 1, 31); // day 30 from Jan 1.
        let r = compute(&i);
        assert!(r.election_filed_within_30_days);
        assert!(r.eligible_for_83i_deferral);
    }

    #[test]
    fn election_window_day_31_violates() {
        let mut i = base();
        i.vesting_or_exercise_date = d(2025, 1, 1);
        i.deferral_election_date = d(2025, 2, 1); // day 31.
        let r = compute(&i);
        assert!(!r.election_filed_within_30_days);
        assert_eq!(r.income_tax_currently_deferred, Decimal::ZERO);
        assert!(r.note.contains("30-day"));
    }

    #[test]
    fn election_before_vesting_disqualifies() {
        let mut i = base();
        i.vesting_or_exercise_date = d(2025, 1, 15);
        i.deferral_election_date = d(2025, 1, 14);
        let r = compute(&i);
        assert!(!r.election_filed_within_30_days);
    }

    #[test]
    fn ipo_trigger_ends_deferral_early() {
        let mut i = base();
        i.stock_became_tradable_date = Some(d(2027, 6, 1));
        i.as_of_date = d(2027, 6, 1);
        let r = compute(&i);
        assert_eq!(r.effective_deferral_end_date, d(2027, 6, 1));
        assert_eq!(r.trigger, DeferralEndTrigger::StockBecameTradable);
    }

    #[test]
    fn revocation_trigger_overrides_5y() {
        let mut i = base();
        i.employee_revoked_date = Some(d(2026, 3, 1));
        i.as_of_date = d(2026, 4, 1);
        let r = compute(&i);
        assert_eq!(r.effective_deferral_end_date, d(2026, 3, 1));
        assert_eq!(r.trigger, DeferralEndTrigger::EmployeeRevoked);
    }

    #[test]
    fn employer_buyback_ends_deferral() {
        let mut i = base();
        i.stock_transferred_to_employer_date = Some(d(2028, 8, 15));
        i.as_of_date = d(2028, 8, 15);
        let r = compute(&i);
        assert_eq!(r.trigger, DeferralEndTrigger::StockTransferredToEmployer);
    }

    #[test]
    fn employee_promoted_to_cfo_mid_deferral_ends_it() {
        let mut i = base();
        i.employee_became_excluded_date = Some(d(2027, 9, 1));
        i.as_of_date = d(2027, 10, 1);
        let r = compute(&i);
        assert_eq!(r.trigger, DeferralEndTrigger::EmployeeBecameExcluded);
    }

    #[test]
    fn earliest_of_multiple_triggers_wins() {
        // 5y max → 2030-01-15. IPO 2026. Revocation 2027. Earliest = IPO.
        let mut i = base();
        i.stock_became_tradable_date = Some(d(2026, 6, 1));
        i.employee_revoked_date = Some(d(2027, 6, 1));
        i.as_of_date = d(2026, 7, 1);
        let r = compute(&i);
        assert_eq!(r.trigger, DeferralEndTrigger::StockBecameTradable);
        assert_eq!(r.effective_deferral_end_date, d(2026, 6, 1));
    }

    #[test]
    fn fica_at_employee_only_rate() {
        // 7.65% employee-only side. $500k × 7.65% = $38,250.
        let mut i = base();
        i.fica_combined_rate_bp = 765;
        let r = compute(&i);
        assert_eq!(r.fica_due_at_vesting, dec!(38_250));
    }

    #[test]
    fn fica_still_due_even_when_employee_excluded() {
        // §3121 FICA is independent of §83(i) eligibility. Excluded
        // employee still owes FICA on the §83(a) inclusion.
        let mut i = base();
        i.employee_is_or_was_ceo_cfo_current_or_10y = true;
        let r = compute(&i);
        assert_eq!(r.fica_due_at_vesting, dec!(76_500));
    }

    #[test]
    fn recognition_year_set_only_when_eligible() {
        let r = compute(&base());
        assert_eq!(r.income_tax_recognition_year, Some(2030));

        let mut i = base();
        i.employee_in_top_4_paid_current_or_10y = true;
        let r2 = compute(&i);
        assert_eq!(r2.income_tax_recognition_year, None);
    }

    #[test]
    fn five_year_endpoint_uses_calendar_months_not_days() {
        // Vest 2024-02-29 + 60 months = 2029-02-28 (Months handles leap).
        let mut i = base();
        i.vesting_or_exercise_date = d(2024, 2, 29);
        i.deferral_election_date = d(2024, 3, 1);
        let r = compute(&i);
        assert_eq!(r.five_year_statutory_end_date, d(2029, 2, 28));
    }

    #[test]
    fn note_describes_eligible_path_with_fica_total() {
        let r = compute(&base());
        assert!(r.note.contains("§83(i) deferral ELIGIBLE"));
        assert!(r.note.contains("$76500"));
        assert!(r.note.contains("FICA"));
    }
}
