//! IRC §280A — Dwelling unit used as a residence (mixed-use property
//! rules) plus §280A(g) "Augusta Rule" tax-free rental.
//!
//! Three IRC §280A classifications drive how a property's income and
//! deductions hit the return:
//!
//!   * **Rental property** — rental use ≥ 15 days AND personal use is
//!     within the IRS threshold (≤ 14 days OR ≤ 10% of fair rental
//!     days, whichever is GREATER). Income and expenses flow to
//!     Schedule E in full, subject to the §469 passive-activity-loss
//!     limit.
//!
//!   * **Mixed-use vacation home** — rental use ≥ 15 days AND personal
//!     use EXCEEDS the threshold. §280A(c)(5) limits deductions to
//!     gross rental income — the property cannot generate a tax loss.
//!     Excess deductions carry forward. Expenses must be allocated
//!     between personal and rental days.
//!
//!   * **Personal residence** — rental use < 15 days. Two outcomes:
//!     - rental days = 0 OR personal use also dominates → not a rental
//!       activity; no rental income reported, no Schedule E.
//!     - rental days 1-14 → **§280A(g) Augusta Rule** — rental income
//!       is **tax-free**, but NO rental deductions are allowed. Most
//!       famously used by homeowners renting to corporations for
//!       board meetings; the corp deducts the rent, the homeowner
//!       excludes it.
//!
//! The 14-day / 10% threshold uses the GREATER of the two — a property
//! rented for 200 days passes if personal use ≤ 20 days (10% of 200),
//! not just ≤ 14 days.
//!
//! Pure compute. Caller passes the days + the year's rental income
//! and expense breakdown; we classify, allocate, and (for vacation
//! homes) apply the §280A(c)(5) deduction ceiling with carry-forward.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    /// Rental property: Schedule E in full, §469 PAL rules apply.
    Rental,
    /// Mixed-use vacation home: §280A(c)(5) deduction-to-income cap.
    VacationHome,
    /// §280A(g) Augusta Rule: rental days 1-14 → tax-free income, no
    /// deductions.
    AugustaRule,
    /// Pure personal residence: no rental activity.
    #[default]
    PersonalResidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280AInput {
    pub fair_rental_days: u32,
    pub personal_use_days: u32,
    /// Gross rental income for the year.
    pub gross_rental_income: Decimal,
    /// Expenses that are deductible REGARDLESS of rental status
    /// (mortgage interest on Schedule A, property tax on Schedule A,
    /// casualty losses). These are NOT capped by §280A(c)(5) — they're
    /// already deductible elsewhere. Allocated to rental portion.
    pub tier_1_expenses_personal_deductible: Decimal,
    /// Operating expenses (insurance, utilities, repairs, management,
    /// supplies, advertising). NOT depreciation. Allocated to rental
    /// portion, then capped by remaining income after tier 1.
    pub tier_2_operating_expenses: Decimal,
    /// Depreciation for the year. Lowest priority — capped by income
    /// after tier 1 and tier 2 absorb. Excess carries forward.
    pub tier_3_depreciation: Decimal,
    /// Suspended deductions carried forward from prior year's §280A
    /// limitation (tier 2 + tier 3 only — tier 1 never suspends).
    pub prior_year_suspended: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section280AResult {
    pub classification: Classification,
    /// 14 OR 10% of fair_rental_days — whichever is GREATER. The
    /// personal-use ceiling for staying in the rental bucket.
    pub personal_use_threshold: u32,
    /// Fraction of expenses allocated to rental use:
    ///   rental_days / (rental_days + personal_use_days).
    /// Zero for non-rental classifications.
    pub rental_allocation_pct: Decimal,
    pub gross_rental_income_reported: Decimal,
    /// Total deductions allowed THIS year (sum of tier 1, capped
    /// tier 2, capped tier 3).
    pub deductions_allowed: Decimal,
    pub tier_1_deductions_allowed: Decimal,
    pub tier_2_deductions_allowed: Decimal,
    pub tier_3_deductions_allowed: Decimal,
    /// Tier 2 + tier 3 deductions disallowed this year (carryforward
    /// pool — tier 1 never suspends).
    pub deductions_suspended_to_next_year: Decimal,
    /// Net income after deductions. Zero or positive — §280A vacation
    /// home cannot generate a net loss.
    pub net_rental_income: Decimal,
    pub note: String,
}

fn personal_use_ceiling(fair_rental_days: u32) -> u32 {
    // Greater of 14 days OR 10% of fair_rental_days. Per IRS Pub 527.
    let ten_pct = (fair_rental_days as f64 * 0.10).floor() as u32;
    14.max(ten_pct)
}

fn classify(input: &Section280AInput, threshold: u32) -> Classification {
    if input.fair_rental_days == 0 {
        return Classification::PersonalResidence;
    }
    if input.fair_rental_days < 15 {
        return Classification::AugustaRule;
    }
    // rental_days >= 15: bucket on personal-use threshold.
    if input.personal_use_days > threshold {
        Classification::VacationHome
    } else {
        Classification::Rental
    }
}

fn allocation_pct(rental: u32, personal: u32) -> Decimal {
    let total = rental + personal;
    if total == 0 {
        return Decimal::ZERO;
    }
    // Decimal::from with division — keep 4 decimal places precision.
    let r = Decimal::from(rental);
    let t = Decimal::from(total);
    (r / t).round_dp(6)
}

pub fn compute(input: &Section280AInput) -> Section280AResult {
    let mut r = Section280AResult {
        personal_use_threshold: personal_use_ceiling(input.fair_rental_days),
        ..Section280AResult::default()
    };
    r.classification = classify(input, r.personal_use_threshold);

    match r.classification {
        Classification::PersonalResidence => {
            r.note = "no rental activity (rental days = 0)".into();
            // Everything else stays zero.
        }
        Classification::AugustaRule => {
            // §280A(g): rental income tax-free, no rental deductions.
            r.gross_rental_income_reported = Decimal::ZERO; // tax-free, not reported
            r.note = format!(
                "§280A(g) Augusta Rule: {} rental days < 15, ${} income tax-free, no deductions",
                input.fair_rental_days, input.gross_rental_income
            );
        }
        Classification::Rental => {
            r.rental_allocation_pct = allocation_pct(input.fair_rental_days, input.personal_use_days);
            r.gross_rental_income_reported = input.gross_rental_income;
            // Full deduction at the rental allocation %.
            let t1 = (input.tier_1_expenses_personal_deductible * r.rental_allocation_pct).round_dp(2);
            let t2 = (input.tier_2_operating_expenses * r.rental_allocation_pct).round_dp(2);
            let t3 = (input.tier_3_depreciation * r.rental_allocation_pct).round_dp(2);
            r.tier_1_deductions_allowed = t1;
            r.tier_2_deductions_allowed = t2 + input.prior_year_suspended; // carryforward releases when not §280A
            r.tier_3_deductions_allowed = t3;
            r.deductions_allowed = t1 + r.tier_2_deductions_allowed + t3;
            r.net_rental_income = r.gross_rental_income_reported - r.deductions_allowed;
            r.note = format!(
                "rental property: {} rental days ≥ 15, {} personal days ≤ {} threshold; full Schedule E (§469 PAL applies separately)",
                input.fair_rental_days, input.personal_use_days, r.personal_use_threshold
            );
        }
        Classification::VacationHome => {
            r.rental_allocation_pct = allocation_pct(input.fair_rental_days, input.personal_use_days);
            r.gross_rental_income_reported = input.gross_rental_income;

            // Tier 1 always allowed at the rental allocation.
            let t1 = (input.tier_1_expenses_personal_deductible * r.rental_allocation_pct).round_dp(2);
            r.tier_1_deductions_allowed = t1;

            // Tier 2: allocated then capped by remaining income after tier 1.
            let t2_full = (input.tier_2_operating_expenses * r.rental_allocation_pct).round_dp(2);
            let t2_pool = t2_full + input.prior_year_suspended;
            let income_after_t1 = (r.gross_rental_income_reported - t1).max(Decimal::ZERO);
            r.tier_2_deductions_allowed = t2_pool.min(income_after_t1);
            let t2_suspended = (t2_pool - r.tier_2_deductions_allowed).max(Decimal::ZERO);

            // Tier 3: allocated then capped by remaining income after tier 1 + tier 2.
            let t3_full = (input.tier_3_depreciation * r.rental_allocation_pct).round_dp(2);
            let income_after_t2 = (income_after_t1 - r.tier_2_deductions_allowed).max(Decimal::ZERO);
            r.tier_3_deductions_allowed = t3_full.min(income_after_t2);
            let t3_suspended = (t3_full - r.tier_3_deductions_allowed).max(Decimal::ZERO);

            r.deductions_allowed = r.tier_1_deductions_allowed
                + r.tier_2_deductions_allowed
                + r.tier_3_deductions_allowed;
            r.deductions_suspended_to_next_year = t2_suspended + t3_suspended;
            r.net_rental_income = (r.gross_rental_income_reported - r.deductions_allowed)
                .max(Decimal::ZERO);
            r.note = format!(
                "§280A vacation home: {} personal days > {} threshold; ${} deductions allowed (capped), ${} suspended",
                input.personal_use_days, r.personal_use_threshold,
                r.deductions_allowed, r.deductions_suspended_to_next_year,
            );
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section280AInput {
        Section280AInput {
            fair_rental_days: 300,
            personal_use_days: 0,
            gross_rental_income: dec!(36000),
            tier_1_expenses_personal_deductible: dec!(12000), // mortgage + property tax
            tier_2_operating_expenses: dec!(8000),
            tier_3_depreciation: dec!(10000),
            prior_year_suspended: Decimal::ZERO,
        }
    }

    #[test]
    fn pure_rental_no_personal_use_full_deduction() {
        let r = compute(&base());
        assert_eq!(r.classification, Classification::Rental);
        assert_eq!(r.rental_allocation_pct, dec!(1.000000));
        assert_eq!(r.deductions_allowed, dec!(30000));
        // Schedule E shows -$6k loss (subject to §469 PAL separately).
        assert_eq!(r.net_rental_income, dec!(6000));
    }

    #[test]
    fn rental_with_personal_use_within_threshold_allocates_proportionally() {
        // 300 rental + 20 personal. Threshold = max(14, 30) = 30. 20 <= 30 → rental.
        // Allocation = 300/320 = 0.9375.
        let mut i = base();
        i.personal_use_days = 20;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Rental);
        assert_eq!(r.personal_use_threshold, 30);
        assert_eq!(r.rental_allocation_pct, dec!(0.937500));
    }

    #[test]
    fn rental_threshold_uses_max_of_14_and_10pct() {
        // 100 rental days → 10% = 10, but 14-day floor applies. Threshold = 14.
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 13;
        let r = compute(&i);
        assert_eq!(r.personal_use_threshold, 14);
        assert_eq!(r.classification, Classification::Rental);
    }

    #[test]
    fn rental_threshold_at_14_exactly_personal_still_rental() {
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 14; // exactly at threshold
        let r = compute(&i);
        assert_eq!(r.classification, Classification::Rental);
    }

    #[test]
    fn rental_threshold_15_personal_days_vacation_home() {
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 15; // 1 day over 14-day threshold
        let r = compute(&i);
        assert_eq!(r.classification, Classification::VacationHome);
    }

    #[test]
    fn rental_300_days_personal_31_vacation_home() {
        // Threshold = max(14, 30) = 30. 31 personal > 30 → vacation home.
        let mut i = base();
        i.personal_use_days = 31;
        let r = compute(&i);
        assert_eq!(r.personal_use_threshold, 30);
        assert_eq!(r.classification, Classification::VacationHome);
    }

    #[test]
    fn vacation_home_deductions_capped_at_income_no_loss() {
        // Vacation home: 100 rental + 30 personal.
        // Threshold = max(14, 10) = 14. 30 > 14 → vacation home.
        // Allocation = 100/130 = 0.769231.
        // Income $36k. Tier 1 = 12000 * 0.769231 = $9230.77.
        // Tier 2 cap = $36k - $9230.77 = $26769.23. Tier 2 alloc = 8000*0.769231 = $6153.85. Allowed in full.
        // Tier 3 cap = $26769.23 - $6153.85 = $20615.38. Tier 3 alloc = 10000*0.769231 = $7692.31. Allowed in full.
        // Plenty of headroom → no suspension this year.
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 30;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::VacationHome);
        assert!(r.net_rental_income >= Decimal::ZERO);
        assert_eq!(r.deductions_suspended_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn vacation_home_low_income_suspends_excess_to_next_year() {
        // Income $5k, lots of expenses. Total deductions capped at $5k income.
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 50; // 50 > 14 → vacation home
        i.gross_rental_income = dec!(5000);
        let r = compute(&i);
        assert_eq!(r.classification, Classification::VacationHome);
        assert_eq!(r.net_rental_income, Decimal::ZERO);
        assert!(r.deductions_suspended_to_next_year > Decimal::ZERO);
        // Allowed = $5k income (cap), so suspended = total alloc - 5000.
        // Allocation = 100/150 = 0.666667. tier_alloc = 12000+8000+10000 = 30000 × 0.666667 = ~20000.
        // Tier 1 = 8000 (always allowed, but tier 1 portion is 12k*0.667=$8000).
        // Wait, tier 1 IS allowed full at allocation — only tier 2 + 3 get suspended.
        // So allowed = $8000 (tier 1) + capped tier 2 (~$0, no income left after tier 1 exceeds income).
        // Net income = max(0, 5000 - allowed). If tier 1 alone is $8000 > $5000, deductions_allowed = $8000.
        // But net_rental_income clamps at 0. Hmm let me think again.
        //
        // Actually tier 1 is ALWAYS deductible regardless of §280A cap because
        // mortgage interest + property tax are already deductible on Schedule A.
        // So tier 1 isn't really "capped" — it just shows up on Schedule E
        // at the rental allocation, and the personal allocation goes to
        // Schedule A. §280A(c)(5) caps tier 2 + 3 at income REMAINING after
        // tier 1. The "no rental loss" rule applies to tier 2 + 3, not tier 1.
        //
        // Our code does this: t1 is allowed in full (no cap), t2 capped by
        // income_after_t1 (which can go negative), t3 by income_after_t2.
        // Net income clamps at 0.
        //
        // In this test: t1 = $8000, t2 cap = $5000 - $8000 = negative → max 0.
        // So t2_allowed = 0, t3_allowed = 0. Total deductions = $8000.
        // Net rental income = max(0, $5000 - $8000) = 0.
        // Suspended = (t2_full + t3_full) - 0 = (8000+10000)*0.667 = $12000.
        assert!(r.tier_1_deductions_allowed > Decimal::ZERO);
        assert_eq!(r.tier_2_deductions_allowed, Decimal::ZERO);
        assert_eq!(r.tier_3_deductions_allowed, Decimal::ZERO);
    }

    #[test]
    fn augusta_rule_14_days_rental_income_tax_free() {
        // 14 rental days → §280A(g): tax-free income, no deductions.
        let mut i = base();
        i.fair_rental_days = 14;
        i.personal_use_days = 351;
        i.gross_rental_income = dec!(20000); // boardroom rental
        let r = compute(&i);
        assert_eq!(r.classification, Classification::AugustaRule);
        assert_eq!(r.gross_rental_income_reported, Decimal::ZERO); // not reported
        assert_eq!(r.deductions_allowed, Decimal::ZERO);
        assert!(r.note.contains("Augusta"));
    }

    #[test]
    fn augusta_rule_boundary_14_days() {
        let mut i = base();
        i.fair_rental_days = 14;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::AugustaRule);
    }

    #[test]
    fn augusta_rule_boundary_15_days_not_augusta() {
        let mut i = base();
        i.fair_rental_days = 15;
        i.personal_use_days = 0;
        let r = compute(&i);
        // 15 rental days, 0 personal → rental (not Augusta).
        assert_eq!(r.classification, Classification::Rental);
    }

    #[test]
    fn personal_residence_zero_rental_days() {
        let mut i = base();
        i.fair_rental_days = 0;
        i.personal_use_days = 365;
        let r = compute(&i);
        assert_eq!(r.classification, Classification::PersonalResidence);
        assert_eq!(r.deductions_allowed, Decimal::ZERO);
    }

    #[test]
    fn vacation_home_prior_suspended_stacks_with_tier_2() {
        let mut i = base();
        i.fair_rental_days = 100;
        i.personal_use_days = 50;
        i.gross_rental_income = dec!(50000); // enough headroom
        i.prior_year_suspended = dec!(15000); // big prior carryforward
        let r = compute(&i);
        assert_eq!(r.classification, Classification::VacationHome);
        // Tier 2 should pick up the prior suspended within remaining
        // income-after-tier-1.
        assert!(r.tier_2_deductions_allowed >= dec!(15000)
            || r.deductions_suspended_to_next_year > Decimal::ZERO);
    }

    #[test]
    fn rental_no_personal_use_zero_threshold_no_panic() {
        let mut i = base();
        i.fair_rental_days = 1;
        i.personal_use_days = 0;
        let r = compute(&i);
        // 1 rental day < 15 → Augusta. Boundary correctness.
        assert_eq!(r.classification, Classification::AugustaRule);
    }

    #[test]
    fn allocation_pct_zero_when_both_days_zero() {
        let pct = allocation_pct(0, 0);
        assert_eq!(pct, Decimal::ZERO);
    }

    #[test]
    fn personal_use_ceiling_uses_greater_of_14_or_10pct() {
        assert_eq!(personal_use_ceiling(100), 14);  // 10% = 10, floor 14
        assert_eq!(personal_use_ceiling(140), 14);  // 10% = 14, tie -> 14
        assert_eq!(personal_use_ceiling(200), 20);  // 10% = 20 > 14
        assert_eq!(personal_use_ceiling(365), 36);  // 10% = 36 > 14
    }
}
