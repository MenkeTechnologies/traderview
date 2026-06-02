//! IRC § 422 — Incentive Stock Options (ISOs). Direct
//! trader companion to section_475c2 (mark-to-market
//! election), section_1411 (NIIT 3.8% on net
//! investment income), section_408a (Roth IRA — ISO
//! qualified-disposition basis adjustment), and the
//! § 56(b)(3) AMT preference adjustment.
//!
//! Trader-critical because traders working at venture-
//! backed companies routinely receive ISO grants:
//! - **Qualified disposition** (2-year-from-grant +
//!   1-year-from-exercise rule) — entire spread taxed
//!   at LONG-TERM CAPITAL GAINS rates (0%/15%/20% +
//!   3.8% NIIT under § 1411); NO ordinary income on
//!   exercise; NO FICA;
//! - **Disqualifying disposition** — spread on
//!   exercise (FMV − exercise price) recharacterized
//!   as ORDINARY COMPENSATION INCOME taxed up to 37%
//!   federal + state; subsequent appreciation taxed
//!   as short-term or long-term capital gain
//!   depending on holding period from exercise;
//! - **AMT preference** (§ 56(b)(3) + § 56(b)(3)(B))
//!   — spread on exercise IS A POSITIVE ADJUSTMENT to
//!   AMT income even on a qualified hold; can trigger
//!   large AMT liability the year of exercise that
//!   creates AMT credit recoverable in future years
//!   under § 53.
//!
//! **§ 422(b) ISO STATUTORY REQUIREMENTS** — option
//! must satisfy ALL six conditions to qualify as ISO:
//! 1. § 422(b)(1) — option granted PURSUANT TO
//!    SHAREHOLDER-APPROVED PLAN within 12 months
//!    before or after board adoption;
//! 2. § 422(b)(2) — option granted within 10 YEARS of
//!    plan adoption or shareholder approval;
//! 3. § 422(b)(3) — option exercise period not more
//!    than 10 YEARS from grant date (5 years for 10%+
//!    shareholders);
//! 4. § 422(b)(4) — option price NOT LESS THAN FMV at
//!    grant (110% of FMV for 10%+ shareholders);
//! 5. § 422(b)(5) — option NOT TRANSFERABLE except by
//!    will or inheritance;
//! 6. § 422(b)(6) — recipient must be EMPLOYEE of
//!    granting corporation, parent, or subsidiary at
//!    grant AND through 3 months before exercise.
//!
//! **§ 422(d) $100,000 ANNUAL LIMIT** — aggregate FMV
//! (measured AT GRANT DATE) of stock for which ISOs
//! are FIRST EXERCISABLE in any calendar year cannot
//! exceed $100,000; any excess is automatically
//! TREATED AS NQSOs from the date of grant (ordinary
//! income on exercise; no AMT preference).
//!
//! **§ 422(a)(1)-(2) QUALIFIED DISPOSITION HOLDING
//! PERIODS**:
//! - 2 YEARS from GRANT DATE;
//! - 1 YEAR from EXERCISE DATE;
//! - BOTH must be satisfied — fails either = DISQUALI-
//!   FYING DISPOSITION.
//!
//! **§ 421(b) DISQUALIFYING DISPOSITION TREATMENT** —
//! when ISO sold before satisfying § 422(a) holding
//! periods:
//! - ORDINARY INCOME = LESSER of (a) FMV at exercise
//!   minus exercise price; OR (b) sale price minus
//!   exercise price (capped at actual gain);
//! - REMAINING GAIN = capital gain (short-term or
//!   long-term depending on holding from exercise);
//! - Employer entitled to corresponding § 162
//!   compensation deduction (lost in qualifying
//!   disposition).
//!
//! **§ 56(b)(3) AMT ADJUSTMENT ON EXERCISE** —
//! spread (FMV at exercise minus exercise price) is
//! POSITIVE AMT adjustment in year of exercise; basis
//! in stock for AMT purposes = FMV at exercise (not
//! exercise price); subsequent sale generates AMT-
//! basis capital gain DIFFERENT from regular-tax
//! capital gain; AMT credit under § 53 recovers
//! over time but slow.
//!
//! **§ 422(c)(2) early disposition reverses § 56(b)(3)
//! adjustment** — if ISO stock SOLD IN SAME YEAR AS
//! EXERCISE (disqualifying disposition), § 56(b)(3)
//! AMT adjustment is REVERSED; no AMT preference
//! applies; ordinary income on disqualifying
//! disposition replaces AMT preference.
//!
//! **Trader-critical fact patterns**:
//! 1. Pre-IPO trader exercises 50,000 ISOs at $1 strike
//!    when FMV $50; $2.45M AMT spread; if holds, owes
//!    ~$686K AMT at 28% AMT rate; AMT credit recovers
//!    over 7-15 years;
//! 2. Same trader sells immediately at $50 (disquali-
//!    fying same-year) — ordinary income $2.45M taxed
//!    at 37% = $906K; § 422(c)(2) reverses AMT
//!    adjustment; ordinary tax replaces AMT;
//! 3. Trader holds 2+1 years, sells at $80 — entire
//!    $79 per share = LTCG at 20% + 3.8% NIIT =
//!    $951K on 50K shares (vs. $1.46M if same-year
//!    disqualifying);
//! 4. § 422(d) exceeded — trader granted $200K ISO
//!    vest in year 1; $100K treats as NQSO from grant;
//!    NQSO portion ordinary income on exercise; no
//!    AMT preference on NQSO portion;
//! 5. § 422(b)(6) — trader leaves company on June 1;
//!    exercises September 1 (more than 3 months
//!    later) — option AUTOMATICALLY converts to NQSO;
//!    ordinary income on exercise.
//!
//! Citations: 26 USC § 422(a)-(d); 26 USC § 421(a)-(b);
//! 26 USC § 56(b)(3); 26 USC § 56(b)(3)(B); 26 USC
//! § 53 (AMT credit); 26 USC § 1411 (NIIT 3.8%);
//! 26 USC § 162 (employer deduction); Treas. Reg.
//! § 1.422-1 through § 1.422-5; Treas. Reg.
//! § 1.421-1 through § 1.421-2; Form 3921 (ISO
//! Exercise Information Statement); IRS Pub. 525;
//! Rev. Rul. 71-52; FTB Publication 1004 (California
//! conformity).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispositionType {
    /// Stock held at year end; no disposition yet.
    NoDisposition,
    /// Both 2-year from grant AND 1-year from exercise
    /// satisfied at sale.
    QualifyingDisposition,
    /// Sale before satisfying § 422(a) holding period.
    DisqualifyingDisposition,
    /// Sale in SAME TAX YEAR as exercise — special
    /// § 422(c)(2) AMT reversal.
    SameYearDisqualifyingDisposition,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section422Input {
    /// FMV at grant date in cents per share.
    pub fmv_at_grant_cents: u64,
    /// Exercise price in cents per share.
    pub exercise_price_cents: u64,
    /// FMV at exercise date in cents per share.
    pub fmv_at_exercise_cents: u64,
    /// Sale price in cents per share (0 if no
    /// disposition).
    pub sale_price_cents: u64,
    /// Number of shares.
    pub shares_count: u64,
    pub disposition: DispositionType,
    /// Aggregate FMV (at grant) of ISOs first exercisable
    /// in this calendar year in dollars; § 422(d) $100K
    /// trigger.
    pub aggregate_iso_grant_fmv_this_year_dollars: u64,
    /// Whether option was granted pursuant to share-
    /// holder-approved plan (§ 422(b)(1)).
    pub shareholder_approved_plan: bool,
    /// Whether option price was not less than FMV at
    /// grant (§ 422(b)(4)).
    pub price_at_least_fmv_at_grant: bool,
    /// Whether recipient was employee through 3 months
    /// before exercise (§ 422(b)(6)).
    pub employee_through_3_months_before_exercise: bool,
    /// Whether option exercise period within 10-year
    /// limit (§ 422(b)(3)).
    pub within_10_year_exercise_period: bool,
    /// Years between grant date and sale.
    pub years_grant_to_sale: u32,
    /// Years between exercise date and sale.
    pub years_exercise_to_sale: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section422Result {
    pub iso_qualifies_under_section_422_b: bool,
    pub portion_subject_to_100k_limit_dollars: u64,
    pub excess_treated_as_nqso_dollars: u64,
    pub holding_periods_satisfied: bool,
    pub amt_preference_cents: u64,
    pub ordinary_income_on_disqualifying_disposition_cents: u64,
    pub long_term_capital_gain_cents: u64,
    pub same_year_disqualifying_reverses_amt: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section422Input) -> Section422Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let statutory_compliant = input.shareholder_approved_plan
        && input.price_at_least_fmv_at_grant
        && input.employee_through_3_months_before_exercise
        && input.within_10_year_exercise_period;

    let (portion_subject_to_100k_limit_dollars, excess_treated_as_nqso_dollars) =
        if input.aggregate_iso_grant_fmv_this_year_dollars > 100_000 {
            (
                100_000u64,
                input.aggregate_iso_grant_fmv_this_year_dollars - 100_000,
            )
        } else {
            (input.aggregate_iso_grant_fmv_this_year_dollars, 0u64)
        };

    let iso_qualifies_under_section_422_b = statutory_compliant;

    let holding_periods_satisfied = input.years_grant_to_sale >= 2
        && input.years_exercise_to_sale >= 1
        && matches!(input.disposition, DispositionType::QualifyingDisposition);

    let spread_at_exercise_cents = input
        .fmv_at_exercise_cents
        .saturating_sub(input.exercise_price_cents);

    let same_year_disqualifying_reverses_amt = matches!(
        input.disposition,
        DispositionType::SameYearDisqualifyingDisposition
    );

    let amt_preference_cents = if same_year_disqualifying_reverses_amt
        || !iso_qualifies_under_section_422_b
    {
        0
    } else {
        spread_at_exercise_cents.saturating_mul(input.shares_count)
    };

    let sale_minus_strike = input.sale_price_cents.saturating_sub(input.exercise_price_cents);
    let lesser_ordinary = spread_at_exercise_cents.min(sale_minus_strike);

    let ordinary_income_on_disqualifying_disposition_cents = if matches!(
        input.disposition,
        DispositionType::DisqualifyingDisposition | DispositionType::SameYearDisqualifyingDisposition
    ) {
        lesser_ordinary.saturating_mul(input.shares_count)
    } else {
        0
    };

    let long_term_capital_gain_cents = if holding_periods_satisfied {
        input
            .sale_price_cents
            .saturating_sub(input.exercise_price_cents)
            .saturating_mul(input.shares_count)
    } else {
        0
    };

    if !input.shareholder_approved_plan {
        failure_reasons.push(
            "26 USC § 422(b)(1) — option NOT GRANTED pursuant to shareholder-approved plan within 12 months before or after board adoption; FAILS ISO statutory requirement; option treated as NQSO".to_string(),
        );
    }
    if !input.price_at_least_fmv_at_grant {
        failure_reasons.push(
            "26 USC § 422(b)(4) — option price LESS THAN FMV at grant (or LESS THAN 110% for 10%+ shareholders); FAILS ISO statutory requirement; option treated as NQSO".to_string(),
        );
    }
    if !input.employee_through_3_months_before_exercise {
        failure_reasons.push(
            "26 USC § 422(b)(6) — recipient NOT EMPLOYEE of granting corporation through 3 months before exercise; AUTOMATIC CONVERSION to NQSO; ordinary income on exercise".to_string(),
        );
    }
    if !input.within_10_year_exercise_period {
        failure_reasons.push(
            "26 USC § 422(b)(3) — option exercise period EXCEEDS 10 YEARS from grant (or 5 YEARS for 10%+ shareholders); FAILS ISO statutory requirement; option treated as NQSO".to_string(),
        );
    }

    if excess_treated_as_nqso_dollars > 0 {
        failure_reasons.push(format!(
            "26 USC § 422(d) — aggregate FMV at grant of ISOs first exercisable in calendar year ({} dollars) EXCEEDS $100,000 limit; ${} treated as NQSO from grant; ordinary income on exercise of excess portion + no AMT preference on NQSO portion",
            input.aggregate_iso_grant_fmv_this_year_dollars,
            excess_treated_as_nqso_dollars
        ));
    }

    if matches!(
        input.disposition,
        DispositionType::DisqualifyingDisposition | DispositionType::SameYearDisqualifyingDisposition
    ) {
        failure_reasons.push(format!(
            "26 USC § 421(b) DISQUALIFYING DISPOSITION — fails § 422(a) 2-year-from-grant + 1-year-from-exercise holding periods; spread on exercise ({} cents/share) recharacterized as ORDINARY COMPENSATION INCOME up to 37% federal; total ordinary income {} cents",
            spread_at_exercise_cents,
            ordinary_income_on_disqualifying_disposition_cents
        ));
    }

    if same_year_disqualifying_reverses_amt {
        failure_reasons.push(
            "26 USC § 422(c)(2) — SAME-YEAR DISQUALIFYING DISPOSITION (exercise and sale in same tax year) REVERSES § 56(b)(3) AMT preference adjustment; no AMT preference applies; ordinary income on disqualifying disposition replaces AMT".to_string(),
        );
    }

    if iso_qualifies_under_section_422_b && amt_preference_cents > 0 {
        failure_reasons.push(format!(
            "26 USC § 56(b)(3) — ISO EXERCISE generates AMT PREFERENCE of {} cents (spread × shares); positive adjustment to AMT income; AMT basis = FMV at exercise; § 53 AMT credit recoverable in future years",
            amt_preference_cents
        ));
    }

    if holding_periods_satisfied {
        failure_reasons.push(format!(
            "26 USC § 421(a) + § 422(a) QUALIFYING DISPOSITION — 2-year-from-grant + 1-year-from-exercise periods SATISFIED; entire spread ({} cents) taxed as LONG-TERM CAPITAL GAIN at 0%/15%/20% + § 1411 NIIT 3.8% (no ordinary income; no FICA)",
            long_term_capital_gain_cents
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 422(b) ISO STATUTORY REQUIREMENTS (6 conditions): (1) granted pursuant to shareholder-approved plan; (2) within 10 years of plan adoption; (3) exercise period ≤ 10 years from grant (5 for 10%+ holders); (4) option price ≥ FMV at grant (110% for 10%+); (5) not transferable except by will/inheritance; (6) employee through 3 months before exercise".to_string(),
        "26 USC § 422(d) $100,000 ANNUAL LIMIT — aggregate FMV (at GRANT DATE) of stock for which ISOs first exercisable in calendar year cannot exceed $100,000; excess treated as NQSO from grant; FMV measured at grant not vesting".to_string(),
        "26 USC § 422(a)(1)-(2) QUALIFIED DISPOSITION HOLDING PERIODS — (1) 2 YEARS from GRANT DATE; (2) 1 YEAR from EXERCISE DATE; BOTH must be satisfied; failing either = disqualifying disposition".to_string(),
        "26 USC § 421(b) DISQUALIFYING DISPOSITION — ordinary income = LESSER of (a) FMV at exercise − exercise price; OR (b) sale price − exercise price; remaining gain = capital gain (ST/LT from exercise); employer gets § 162 compensation deduction".to_string(),
        "26 USC § 56(b)(3) AMT ADJUSTMENT — spread (FMV exercise − exercise price) is POSITIVE AMT adjustment in year of exercise; AMT basis = FMV at exercise (NOT exercise price); subsequent sale generates AMT-basis capital gain DIFFERENT from regular-tax capital gain; 28% AMT rate above exemption".to_string(),
        "26 USC § 53 AMT CREDIT — AMT paid in ISO exercise year creates MINIMUM TAX CREDIT recoverable against REGULAR tax in future years; limited to excess of regular tax over tentative minimum tax; recovery often slow (7-15+ years)".to_string(),
        "26 USC § 422(c)(2) SAME-YEAR EARLY DISPOSITION REVERSAL — if ISO stock SOLD IN SAME YEAR AS EXERCISE (disqualifying), § 56(b)(3) AMT adjustment REVERSED; no AMT preference applies; ordinary income on disqualifying disposition replaces AMT preference".to_string(),
        "26 USC § 1411 NIIT 3.8% — qualifying disposition long-term capital gain INCLUDED in net investment income; 3.8% NIIT applies above MAGI thresholds ($200K single / $250K MFJ); disqualifying disposition ORDINARY INCOME portion EXCLUDED from NII (wages/compensation excluded)".to_string(),
        "Trader-critical fact patterns: (1) pre-IPO 50K ISOs $1 strike $50 FMV = $2.45M AMT spread → ~$686K AMT at 28%; (2) same trader sells immediately disqualifying = ordinary income $2.45M at 37% = $906K but § 422(c)(2) reverses AMT; (3) holds 2+1 years sells $80 = entire $79/share LTCG at 23.8% = $951K vs $1.46M disqualifying; (4) § 422(d) $200K ISO vest year 1 — $100K NQSO portion (no AMT); (5) trader leaves June 1 exercises Sept 1 → § 422(b)(6) auto NQSO".to_string(),
        "Form 3921 — ISO Exercise Information Statement; employer must furnish to employee by January 31 of year after exercise; reports grant date, exercise date, exercise price, FMV at exercise, shares acquired; trader must keep for life of stock to track AMT-basis vs regular-basis split".to_string(),
        "California conformity (FTB Publication 1004) — CA conforms to federal § 422 with modifications; CA AMT separately computed under R&TC § 17062; CA AMT rate 7%; disqualifying disposition ordinary income subject to CA wage tax up to 13.3%".to_string(),
        "Companion to section_475c2 (mark-to-market for trader stock) + section_1411 (NIIT 3.8% on qualifying disposition) + section_408a (Roth IRA — but ISOs cannot be held in IRA) + section_53 (AMT credit recovery) + section_56 (AMT preference items)".to_string(),
    ];

    Section422Result {
        iso_qualifies_under_section_422_b,
        portion_subject_to_100k_limit_dollars,
        excess_treated_as_nqso_dollars,
        holding_periods_satisfied,
        amt_preference_cents,
        ordinary_income_on_disqualifying_disposition_cents,
        long_term_capital_gain_cents,
        same_year_disqualifying_reverses_amt,
        failure_reasons,
        citation: "26 USC § 422(a)-(d); 26 USC § 421(a)-(b); 26 USC § 56(b)(3); 26 USC § 56(b)(3)(B); 26 USC § 53 (AMT credit); 26 USC § 1411 (NIIT 3.8%); 26 USC § 162 (employer deduction); Treas. Reg. § 1.422-1 through § 1.422-5; Treas. Reg. § 1.421-1 through § 1.421-2; Form 3921 (ISO Exercise Information Statement); IRS Pub. 525; Rev. Rul. 71-52; FTB Publication 1004 (California conformity)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iso_compliant_holding() -> Section422Input {
        Section422Input {
            fmv_at_grant_cents: 100,
            exercise_price_cents: 100,
            fmv_at_exercise_cents: 5000,
            sale_price_cents: 8000,
            shares_count: 50_000,
            disposition: DispositionType::QualifyingDisposition,
            aggregate_iso_grant_fmv_this_year_dollars: 50_000,
            shareholder_approved_plan: true,
            price_at_least_fmv_at_grant: true,
            employee_through_3_months_before_exercise: true,
            within_10_year_exercise_period: true,
            years_grant_to_sale: 3,
            years_exercise_to_sale: 2,
        }
    }

    #[test]
    fn qualifying_disposition_ltcg_only() {
        let r = check(&iso_compliant_holding());
        assert!(r.iso_qualifies_under_section_422_b);
        assert!(r.holding_periods_satisfied);
        assert_eq!(r.ordinary_income_on_disqualifying_disposition_cents, 0);
        assert_eq!(r.long_term_capital_gain_cents, 7900 * 50_000);
    }

    #[test]
    fn qualifying_disposition_amt_preference_on_exercise() {
        let r = check(&iso_compliant_holding());
        assert_eq!(r.amt_preference_cents, 4900 * 50_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 56(b)(3)")
            && f.contains("AMT PREFERENCE")));
    }

    #[test]
    fn disqualifying_disposition_ordinary_income() {
        let mut i = iso_compliant_holding();
        i.disposition = DispositionType::DisqualifyingDisposition;
        i.years_grant_to_sale = 1;
        i.years_exercise_to_sale = 0;
        let r = check(&i);
        assert!(!r.holding_periods_satisfied);
        assert_eq!(r.ordinary_income_on_disqualifying_disposition_cents, 4900 * 50_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 421(b) DISQUALIFYING DISPOSITION")));
    }

    #[test]
    fn disqualifying_lesser_of_rule_when_sale_below_fmv_exercise() {
        let mut i = iso_compliant_holding();
        i.disposition = DispositionType::DisqualifyingDisposition;
        i.sale_price_cents = 3000;
        i.years_grant_to_sale = 1;
        i.years_exercise_to_sale = 0;
        let r = check(&i);
        assert_eq!(r.ordinary_income_on_disqualifying_disposition_cents, 2900 * 50_000);
    }

    #[test]
    fn same_year_disqualifying_reverses_amt() {
        let mut i = iso_compliant_holding();
        i.disposition = DispositionType::SameYearDisqualifyingDisposition;
        let r = check(&i);
        assert!(r.same_year_disqualifying_reverses_amt);
        assert_eq!(r.amt_preference_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(c)(2)")
            && f.contains("REVERSES")));
    }

    #[test]
    fn one_hundred_k_limit_no_excess() {
        let mut i = iso_compliant_holding();
        i.aggregate_iso_grant_fmv_this_year_dollars = 100_000;
        let r = check(&i);
        assert_eq!(r.portion_subject_to_100k_limit_dollars, 100_000);
        assert_eq!(r.excess_treated_as_nqso_dollars, 0);
    }

    #[test]
    fn one_hundred_k_limit_excess_treated_as_nqso() {
        let mut i = iso_compliant_holding();
        i.aggregate_iso_grant_fmv_this_year_dollars = 200_000;
        let r = check(&i);
        assert_eq!(r.portion_subject_to_100k_limit_dollars, 100_000);
        assert_eq!(r.excess_treated_as_nqso_dollars, 100_000);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(d)")
            && f.contains("$100,000 limit")
            && f.contains("$100000 treated as NQSO")));
    }

    #[test]
    fn no_shareholder_plan_iso_fails() {
        let mut i = iso_compliant_holding();
        i.shareholder_approved_plan = false;
        let r = check(&i);
        assert!(!r.iso_qualifies_under_section_422_b);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(b)(1)")
            && f.contains("NQSO")));
    }

    #[test]
    fn price_below_fmv_iso_fails() {
        let mut i = iso_compliant_holding();
        i.price_at_least_fmv_at_grant = false;
        let r = check(&i);
        assert!(!r.iso_qualifies_under_section_422_b);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(b)(4)")
            && f.contains("LESS THAN FMV")));
    }

    #[test]
    fn left_company_more_than_3_months_iso_fails() {
        let mut i = iso_compliant_holding();
        i.employee_through_3_months_before_exercise = false;
        let r = check(&i);
        assert!(!r.iso_qualifies_under_section_422_b);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(b)(6)")
            && f.contains("AUTOMATIC CONVERSION to NQSO")));
    }

    #[test]
    fn exercise_period_over_10_years_fails() {
        let mut i = iso_compliant_holding();
        i.within_10_year_exercise_period = false;
        let r = check(&i);
        assert!(!r.iso_qualifies_under_section_422_b);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 422(b)(3)")));
    }

    #[test]
    fn holding_periods_truth_table_boundaries() {
        // 2+1 years satisfied
        let mut a = iso_compliant_holding();
        a.years_grant_to_sale = 2;
        a.years_exercise_to_sale = 1;
        assert!(check(&a).holding_periods_satisfied);

        // 1 year from grant — fails
        let mut b = iso_compliant_holding();
        b.years_grant_to_sale = 1;
        b.years_exercise_to_sale = 1;
        b.disposition = DispositionType::QualifyingDisposition;
        assert!(!check(&b).holding_periods_satisfied);

        // 2 years from grant but 0 years exercise — fails
        let mut c = iso_compliant_holding();
        c.years_grant_to_sale = 2;
        c.years_exercise_to_sale = 0;
        c.disposition = DispositionType::QualifyingDisposition;
        assert!(!check(&c).holding_periods_satisfied);
    }

    #[test]
    fn no_disposition_amt_preference_only() {
        let mut i = iso_compliant_holding();
        i.disposition = DispositionType::NoDisposition;
        i.sale_price_cents = 0;
        let r = check(&i);
        assert!(r.amt_preference_cents > 0);
        assert!(!r.holding_periods_satisfied);
        assert_eq!(r.ordinary_income_on_disqualifying_disposition_cents, 0);
    }

    #[test]
    fn iso_fails_statutory_no_amt_preference() {
        let mut i = iso_compliant_holding();
        i.shareholder_approved_plan = false;
        let r = check(&i);
        assert_eq!(r.amt_preference_cents, 0);
    }

    #[test]
    fn disposition_truth_table_four_cells() {
        for (disp, expect_dq_income, expect_ltcg) in [
            (DispositionType::NoDisposition, false, false),
            (DispositionType::QualifyingDisposition, false, true),
            (DispositionType::DisqualifyingDisposition, true, false),
            (DispositionType::SameYearDisqualifyingDisposition, true, false),
        ] {
            let mut i = iso_compliant_holding();
            i.disposition = disp;
            if matches!(disp, DispositionType::DisqualifyingDisposition | DispositionType::SameYearDisqualifyingDisposition) {
                i.years_grant_to_sale = 1;
                i.years_exercise_to_sale = 0;
            }
            let r = check(&i);
            assert_eq!(
                r.ordinary_income_on_disqualifying_disposition_cents > 0,
                expect_dq_income,
                "disp={:?}", disp
            );
            assert_eq!(
                r.long_term_capital_gain_cents > 0,
                expect_ltcg,
                "disp={:?}", disp
            );
        }
    }

    #[test]
    fn same_year_uniquely_reverses_amt_invariant() {
        let mut same_year = iso_compliant_holding();
        same_year.disposition = DispositionType::SameYearDisqualifyingDisposition;
        assert!(check(&same_year).same_year_disqualifying_reverses_amt);

        let mut next_year = iso_compliant_holding();
        next_year.disposition = DispositionType::DisqualifyingDisposition;
        next_year.years_grant_to_sale = 1;
        assert!(!check(&next_year).same_year_disqualifying_reverses_amt);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&iso_compliant_holding());
        assert!(r.citation.contains("§ 422(a)-(d)"));
        assert!(r.citation.contains("§ 421(a)-(b)"));
        assert!(r.citation.contains("§ 56(b)(3)"));
        assert!(r.citation.contains("§ 53 (AMT credit)"));
        assert!(r.citation.contains("§ 1411 (NIIT 3.8%)"));
        assert!(r.citation.contains("§ 162 (employer deduction)"));
        assert!(r.citation.contains("Treas. Reg. § 1.422-1 through § 1.422-5"));
        assert!(r.citation.contains("Treas. Reg. § 1.421-1 through § 1.421-2"));
        assert!(r.citation.contains("Form 3921"));
        assert!(r.citation.contains("Rev. Rul. 71-52"));
        assert!(r.citation.contains("FTB Publication 1004"));
    }

    #[test]
    fn note_pins_subsection_b_six_requirements() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 422(b)")
            && n.contains("(6 conditions)")
            && n.contains("shareholder-approved")
            && n.contains("110% for 10%+")
            && n.contains("3 months before exercise")));
    }

    #[test]
    fn note_pins_subsection_d_100k_limit() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 422(d) $100,000 ANNUAL LIMIT")
            && n.contains("GRANT DATE")
            && n.contains("treated as NQSO")));
    }

    #[test]
    fn note_pins_subsection_a_holding_periods() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 422(a)(1)-(2)")
            && n.contains("2 YEARS from GRANT DATE")
            && n.contains("1 YEAR from EXERCISE DATE")));
    }

    #[test]
    fn note_pins_section_421b_disqualifying_lesser_rule() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 421(b) DISQUALIFYING DISPOSITION")
            && n.contains("LESSER of")
            && n.contains("§ 162 compensation deduction")));
    }

    #[test]
    fn note_pins_section_56b3_amt_adjustment() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 56(b)(3) AMT ADJUSTMENT")
            && n.contains("AMT basis = FMV at exercise")
            && n.contains("28% AMT rate")));
    }

    #[test]
    fn note_pins_section_53_amt_credit() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 53 AMT CREDIT")
            && n.contains("MINIMUM TAX CREDIT")));
    }

    #[test]
    fn note_pins_section_422c2_same_year_reversal() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 422(c)(2)")
            && n.contains("SAME-YEAR EARLY DISPOSITION REVERSAL")));
    }

    #[test]
    fn note_pins_section_1411_niit_3_8() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 1411 NIIT 3.8%")
            && n.contains("$200K single")
            && n.contains("$250K MFJ")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-critical fact patterns")
            && n.contains("50K ISOs $1 strike $50 FMV")
            && n.contains("$686K AMT")
            && n.contains("LTCG at 23.8%")));
    }

    #[test]
    fn note_pins_form_3921() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("Form 3921")
            && n.contains("January 31")
            && n.contains("AMT-basis vs regular-basis")));
    }

    #[test]
    fn note_pins_california_conformity() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("FTB Publication 1004")
            && n.contains("R&TC § 17062")
            && n.contains("CA AMT rate 7%")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&iso_compliant_holding());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to section_475c2")
            && n.contains("section_1411")
            && n.contains("section_53")
            && n.contains("section_56")));
    }

    #[test]
    fn iso_uniquely_generates_amt_preference_invariant() {
        let mut iso = iso_compliant_holding();
        iso.disposition = DispositionType::NoDisposition;
        let r_iso = check(&iso);
        assert!(r_iso.amt_preference_cents > 0);

        let mut nqso = iso_compliant_holding();
        nqso.disposition = DispositionType::NoDisposition;
        nqso.shareholder_approved_plan = false;
        let r_nqso = check(&nqso);
        assert_eq!(r_nqso.amt_preference_cents, 0);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = iso_compliant_holding();
        i.fmv_at_exercise_cents = u64::MAX;
        i.shares_count = u64::MAX;
        let r = check(&i);
        let _ = r.amt_preference_cents;
    }
}
