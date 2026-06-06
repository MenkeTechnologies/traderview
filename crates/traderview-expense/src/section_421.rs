//! IRC § 421 — General rules for treatment of stock
//! issued under qualified statutory stock options.
//! The foundational provision for both incentive stock
//! options (ISOs) under § 422 and employee stock
//! purchase plans (ESPPs) under § 423: no income on
//! grant, no income on exercise (for regular tax
//! purposes), and capital-gain treatment on qualifying
//! disposition. Direct trader-employee companion to
//! section_422 (ISO definitional rules), section_423
//! (ESPP definitional rules), section_409a (NQDC),
//! section_83 (property transferred for services),
//! section_1042 (ESOP rollover — iter 480), section_4978
//! (ESOP recapture — iter 484), section_1366 (S-corp
//! pass-through — iter 488), section_1377 (S-corp
//! definitions — iter 490).
//!
//! § 421(a) GENERAL RULE: if a share of stock is
//! transferred to an individual pursuant to the
//! exercise of a STATUTORY OPTION (defined under § 422
//! or § 423):
//! 1. § 421(a)(1) — NO INCOME at time of exercise
//!    with respect to the transfer (for regular income
//!    tax purposes)
//! 2. § 421(a)(2) — NO § 162 DEDUCTION allowed to the
//!    employer corporation
//! 3. § 421(a)(3) — no amount other than the price paid
//!    under the option shall be considered as
//!    received by any of such corporations for the
//!    share
//!
//! § 421(b) DISQUALIFYING DISPOSITION: if individual
//! DISPOSES of share BEFORE the qualifying-disposition
//! holding period is satisfied:
//! 1. INCREASE in fair market value over option price
//!    at time of exercise is treated as COMPENSATION
//!    (ordinary income) in the taxable year of
//!    disposition
//! 2. § 162 deduction is allowed to employer
//! 3. Additional gain (FMV at disposition - FMV at
//!    exercise) treated as capital gain (long-term or
//!    short-term per § 1222 holding period from
//!    exercise date)
//!
//! ISO qualifying-disposition holding requirements per
//! § 422(a)(1):
//! 1. NO DISPOSITION of share within 2 YEARS from the
//!    date of GRANT of the option
//! 2. NO DISPOSITION of share within 1 YEAR after the
//!    date of TRANSFER (exercise) of the share to the
//!    employee
//! 3. Both periods must be satisfied for qualifying
//!    disposition
//!
//! ESPP qualifying-disposition holding requirements per
//! § 423(a)(1):
//! 1. NO DISPOSITION within 2 YEARS from date of GRANT
//! 2. NO DISPOSITION within 1 YEAR from date of
//!    TRANSFER
//!
//! Employment requirement per § 422(a)(2): individual
//! must be EMPLOYEE of granting corporation (or related
//! corp under § 424(e)/(f)) at all times during the
//! period beginning on date of grant and ending on day
//! 3 MONTHS BEFORE the date of exercise. Death extends
//! to date of death. Disability extends to 12 months.
//!
//! AMT preference per § 56(b)(3): spread between FMV at
//! exercise and option price IS an AMT preference item
//! for the year of exercise — creates "phantom AMT
//! income" before the share is sold. Source of major
//! trader-employee AMT planning concerns when exercising
//! deep-in-the-money ISOs.
//!
//! Information reporting per § 6039: corporation must
//! file Form 3921 with IRS and provide statement to
//! employee for each ISO exercise + Form 3922 for ESPP
//! transfer of stock acquired pursuant to qualifying
//! ESPP option.
//!
//! Coordination with § 1042 (iter 480 ESOP rollover) +
//! § 4978 (iter 484 ESOP recapture): securities received
//! by exercise of ISO under § 422 are NOT 'qualified
//! securities' eligible for § 1042 ESOP rollover treatment
//! per § 1042(c)(1)(B) cross-reference. Trader-founders
//! considering ESOP exit must hold ISO-exercised shares
//! and § 422 stock separately.
//!
//! Coordination with § 83: § 421 statutory-option
//! treatment OVERRIDES § 83 § 83 property-transferred-
//! for-services treatment for qualifying ISO/ESPP
//! exercises. If § 422 or § 423 requirements fail,
//! § 83 controls (immediate ordinary income at
//! exercise).
//!
//! Trader-employee critical because (1) § 421 is the
//! statutory backbone of preferential tax treatment for
//! employee stock options — without it, all option
//! exercises would generate immediate ordinary income;
//! (2) AMT preference under § 56(b)(3) creates "phantom
//! income" risk when exercising deep-in-the-money ISOs
//! without selling — major source of trader-employee
//! tax surprises during dot-com bust + crypto bust
//! eras; (3) disqualifying disposition is INTENTIONAL
//! tax-planning tool when seeking ordinary-income loss
//! deduction (rare but useful for charitable strategies);
//! (4) 2-year/1-year ISO holding period combined with
//! 12-month long-term capital gain holding period
//! creates effective minimum 2-year hold from grant to
//! qualify for both preferential ISO treatment AND
//! long-term capital gain rate; (5) employment
//! requirement under § 422(a)(2) creates termination-
//! exercise window pressure (90 days post-termination
//! exercise window).
//!
//! Authority: 26 U.S.C. § 421; § 421(a)(1); § 421(a)(2);
//! § 421(a)(3); § 421(b); § 422 (incentive stock
//! options); § 422(a)(1) (ISO holding requirements);
//! § 422(a)(2) (employment requirement); § 423 (ESPP);
//! § 423(a)(1) (ESPP holding requirements); § 424(e)/(f)
//! (parent/subsidiary corporation definitions); § 56(b)(3)
//! (AMT preference for ISO); § 162 (employer deduction);
//! § 83 (property transferred for services); § 1042
//! (ESOP rollover — iter 480); § 1222 (capital gain
//! holding period); § 6039 (information reporting);
//! Form 3921 (ISO exercise); Form 3922 (ESPP transfer);
//! 26 C.F.R. § 1.421-1; 26 C.F.R. § 1.421-2; 26 C.F.R.
//! § 1.421-7; 26 C.F.R. § 1.421-8; 26 C.F.R. § 1.422-1;
//! Tax Reform Act of 1976 + Economic Recovery Tax Act
//! of 1981 + American Jobs Creation Act of 2004 —
//! § 421 framework evolution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionType {
    IncentiveStockOptionSection422,
    EmployeeStockPurchasePlanSection423,
    NonqualifiedStockOption, // not covered by § 421
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Grant,
    Exercise,
    Disposition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub option_type: OptionType,
    pub event_type: EventType,
    pub months_since_grant: u32,
    pub months_since_exercise: u32,
    pub option_exercise_price_cents: u64,
    pub fmv_at_exercise_cents: u64,
    pub fmv_at_disposition_cents: u64,
    pub was_employee_throughout_grant_to_exercise_minus_3_months: bool,
    pub employee_died_or_became_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Section421aGrantNoIncome,
    Section421aExerciseNoIncome,
    Section421aQualifyingDispositionCapitalGain,
    Section421bDisqualifyingDispositionCompensation,
    EmploymentRequirementFailed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub ordinary_compensation_income_cents: u64,
    pub capital_gain_cents: u64,
    pub amt_preference_at_exercise_cents: u64,
    pub notes: Vec<String>,
}

pub const ISO_GRANT_HOLDING_MONTHS: u32 = 24; // 2 years
pub const ISO_EXERCISE_HOLDING_MONTHS: u32 = 12; // 1 year

pub type Section421Input = Input;
pub type Section421Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 421(a) GENERAL RULE: if share of stock transferred to individual pursuant to exercise of STATUTORY OPTION (§ 422 ISO or § 423 ESPP): (1) § 421(a)(1) NO INCOME at time of exercise for regular income tax purposes; (2) § 421(a)(2) NO § 162 DEDUCTION to employer; (3) § 421(a)(3) only option price considered as received by corporation.".to_string(),
        "§ 421(b) DISQUALIFYING DISPOSITION: if individual disposes of share BEFORE qualifying-disposition holding period satisfied, INCREASE in FMV over option price at time of exercise treated as COMPENSATION (ordinary income) in year of disposition; § 162 deduction allowed to employer; additional gain (FMV at disposition - FMV at exercise) treated as capital gain per § 1222 holding period from exercise date.".to_string(),
        "§ 422(a)(1) ISO qualifying-disposition holding requirements: NO DISPOSITION within 2 YEARS from GRANT date AND NO DISPOSITION within 1 YEAR from TRANSFER (exercise) date. BOTH periods must be satisfied for qualifying disposition.".to_string(),
        "§ 422(a)(2) employment requirement: individual must be EMPLOYEE of granting corporation (or related corp under § 424(e)/(f)) at all times during period beginning on date of grant and ending on day 3 MONTHS BEFORE date of exercise. DEATH extends to date of death. DISABILITY extends to 12 months.".to_string(),
        "§ 56(b)(3) AMT PREFERENCE: spread between FMV at exercise and option price IS an AMT preference item for year of exercise — creates 'phantom AMT income' before share is sold. Major trader-employee AMT planning concern when exercising deep-in-the-money ISOs without immediate disposition.".to_string(),
        "§ 6039 information reporting: corporation must file Form 3921 with IRS and provide statement to employee for each ISO exercise + Form 3922 for ESPP transfer of stock acquired pursuant to qualifying ESPP option.".to_string(),
        "Coordination with § 1042 (iter 480 ESOP rollover): securities received by exercise of ISO under § 422 are NOT 'qualified securities' eligible for § 1042 ESOP rollover per § 1042(c)(1)(B) cross-reference. Trader-founders considering ESOP exit must hold ISO-exercised shares + § 422 stock separately.".to_string(),
        "Coordination with § 83: § 421 statutory-option treatment OVERRIDES § 83 property-transferred-for-services treatment for qualifying ISO/ESPP exercises. If § 422 or § 423 requirements fail, § 83 controls (immediate ordinary income at exercise).".to_string(),
        "Companion: section_422 (ISO definitional rules), section_423 (ESPP definitional rules), section_409a (NQDC), section_83, section_1042 (iter 480 ESOP rollover), section_4978 (iter 484 ESOP recapture), section_1366 (iter 488 S-corp pass-thru), section_1377 (iter 490 S-corp definitions); also references § 56(b)(3) + § 162 + § 1222 + § 6039 + Form 3921 + Form 3922.".to_string(),
    ];

    if matches!(input.option_type, OptionType::NonqualifiedStockOption) {
        let mut n = notes;
        n.push("Nonqualified stock option (NQSO) — NOT covered by § 421 statutory-option treatment; § 83 controls with immediate ordinary income at exercise (spread between FMV and option price taxed as compensation).".to_string());
        return Output {
            severity: Severity::NotApplicable,
            ordinary_compensation_income_cents: 0,
            capital_gain_cents: 0,
            amt_preference_at_exercise_cents: 0,
            notes: n,
        };
    }

    // Check employment requirement
    let employment_ok = input.was_employee_throughout_grant_to_exercise_minus_3_months
        || input.employee_died_or_became_disabled;
    if !employment_ok
        && matches!(
            input.event_type,
            EventType::Exercise | EventType::Disposition
        )
    {
        let mut n = notes;
        n.push("§ 422(a)(2) employment requirement FAILED: individual was not employee of granting corporation (or related corp under § 424(e)/(f)) at all times during period beginning on grant date and ending 3 months before exercise date. Death or disability extension does not apply. Treats option as NQSO under § 83 with immediate ordinary income at exercise.".to_string());
        return Output {
            severity: Severity::EmploymentRequirementFailed,
            ordinary_compensation_income_cents: input
                .fmv_at_exercise_cents
                .saturating_sub(input.option_exercise_price_cents),
            capital_gain_cents: 0,
            amt_preference_at_exercise_cents: 0,
            notes: n,
        };
    }

    match input.event_type {
        EventType::Grant => {
            let mut n = notes;
            n.push("§ 421(a) Grant event: NO INCOME at time of grant of statutory option. No reporting on Form W-2; no Form 3921/3922 yet.".to_string());
            Output {
                severity: Severity::Section421aGrantNoIncome,
                ordinary_compensation_income_cents: 0,
                capital_gain_cents: 0,
                amt_preference_at_exercise_cents: 0,
                notes: n,
            }
        }
        EventType::Exercise => {
            let amt_preference = input
                .fmv_at_exercise_cents
                .saturating_sub(input.option_exercise_price_cents);
            let mut n = notes;
            n.push(format!(
                "§ 421(a)(1) Exercise event: NO INCOME at time of exercise for regular income tax purposes. § 56(b)(3) AMT preference: ${}.{:02} (FMV ${}.{:02} minus option price ${}.{:02}). Form 3921 (ISO) or Form 3922 (ESPP) reporting required under § 6039.",
                amt_preference / 100, amt_preference % 100,
                input.fmv_at_exercise_cents / 100, input.fmv_at_exercise_cents % 100,
                input.option_exercise_price_cents / 100, input.option_exercise_price_cents % 100
            ));
            Output {
                severity: Severity::Section421aExerciseNoIncome,
                ordinary_compensation_income_cents: 0,
                capital_gain_cents: 0,
                amt_preference_at_exercise_cents: amt_preference,
                notes: n,
            }
        }
        EventType::Disposition => {
            let two_year_grant_satisfied = input.months_since_grant >= ISO_GRANT_HOLDING_MONTHS;
            let one_year_exercise_satisfied =
                input.months_since_exercise >= ISO_EXERCISE_HOLDING_MONTHS;
            let qualifying = two_year_grant_satisfied && one_year_exercise_satisfied;

            let amt_preference = input
                .fmv_at_exercise_cents
                .saturating_sub(input.option_exercise_price_cents);

            if qualifying {
                let capital_gain = input
                    .fmv_at_disposition_cents
                    .saturating_sub(input.option_exercise_price_cents);
                let mut n = notes;
                n.push(format!(
                    "§ 421(a) QUALIFYING DISPOSITION: both 2-year grant and 1-year exercise holding periods satisfied ({} months since grant + {} months since exercise). Entire gain ${}.{:02} (FMV at disposition ${}.{:02} - option price ${}.{:02}) treated as LONG-TERM CAPITAL GAIN per § 1222.",
                    input.months_since_grant, input.months_since_exercise,
                    capital_gain / 100, capital_gain % 100,
                    input.fmv_at_disposition_cents / 100, input.fmv_at_disposition_cents % 100,
                    input.option_exercise_price_cents / 100, input.option_exercise_price_cents % 100
                ));
                Output {
                    severity: Severity::Section421aQualifyingDispositionCapitalGain,
                    ordinary_compensation_income_cents: 0,
                    capital_gain_cents: capital_gain,
                    amt_preference_at_exercise_cents: amt_preference,
                    notes: n,
                }
            } else {
                let compensation = input
                    .fmv_at_exercise_cents
                    .saturating_sub(input.option_exercise_price_cents);
                let additional_capital_gain = input
                    .fmv_at_disposition_cents
                    .saturating_sub(input.fmv_at_exercise_cents);
                let mut n = notes;
                let reason = if !two_year_grant_satisfied && !one_year_exercise_satisfied {
                    "BOTH 2-year grant and 1-year exercise holding periods failed"
                } else if !two_year_grant_satisfied {
                    "2-year grant holding period failed"
                } else {
                    "1-year exercise holding period failed"
                };
                n.push(format!(
                    "§ 421(b) DISQUALIFYING DISPOSITION: {} ({} months since grant + {} months since exercise). FMV-over-option-price spread ${}.{:02} treated as COMPENSATION (ordinary income); additional FMV-at-disposition-over-FMV-at-exercise gain ${}.{:02} treated as capital gain per § 1222 holding period from exercise date.",
                    reason,
                    input.months_since_grant, input.months_since_exercise,
                    compensation / 100, compensation % 100,
                    additional_capital_gain / 100, additional_capital_gain % 100
                ));
                Output {
                    severity: Severity::Section421bDisqualifyingDispositionCompensation,
                    ordinary_compensation_income_cents: compensation,
                    capital_gain_cents: additional_capital_gain,
                    amt_preference_at_exercise_cents: amt_preference,
                    notes: n,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            option_type: OptionType::IncentiveStockOptionSection422,
            event_type: EventType::Disposition,
            months_since_grant: 30,
            months_since_exercise: 15,
            option_exercise_price_cents: 10_00,
            fmv_at_exercise_cents: 50_00,
            fmv_at_disposition_cents: 100_00,
            was_employee_throughout_grant_to_exercise_minus_3_months: true,
            employee_died_or_became_disabled: false,
        }
    }

    #[test]
    fn nqso_not_applicable() {
        let mut i = baseline();
        i.option_type = OptionType::NonqualifiedStockOption;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn grant_event_no_income() {
        let mut i = baseline();
        i.event_type = EventType::Grant;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section421aGrantNoIncome);
        assert_eq!(out.ordinary_compensation_income_cents, 0);
    }

    #[test]
    fn exercise_event_no_income_amt_preference_computed() {
        let mut i = baseline();
        i.event_type = EventType::Exercise;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Section421aExerciseNoIncome);
        assert_eq!(out.amt_preference_at_exercise_cents, 40_00); // $50-$10
        assert_eq!(out.ordinary_compensation_income_cents, 0);
    }

    #[test]
    fn qualifying_disposition_full_capital_gain() {
        let i = baseline(); // 30mo > 24, 15mo > 12 → qualifying
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );
        // capital gain = $100 - $10 = $90
        assert_eq!(out.capital_gain_cents, 90_00);
        assert_eq!(out.ordinary_compensation_income_cents, 0);
    }

    #[test]
    fn disqualifying_disposition_2_year_grant_fails() {
        let mut i = baseline();
        i.months_since_grant = 20; // < 24
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421bDisqualifyingDispositionCompensation
        );
        // compensation = $50 - $10 = $40
        assert_eq!(out.ordinary_compensation_income_cents, 40_00);
        // capital gain = $100 - $50 = $50
        assert_eq!(out.capital_gain_cents, 50_00);
    }

    #[test]
    fn disqualifying_disposition_1_year_exercise_fails() {
        let mut i = baseline();
        i.months_since_grant = 30;
        i.months_since_exercise = 8; // < 12
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421bDisqualifyingDispositionCompensation
        );
        assert_eq!(out.ordinary_compensation_income_cents, 40_00);
    }

    #[test]
    fn disqualifying_disposition_both_periods_fail() {
        let mut i = baseline();
        i.months_since_grant = 20;
        i.months_since_exercise = 8;
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("BOTH 2-year grant and 1-year exercise holding periods failed"));
    }

    #[test]
    fn exactly_2_year_grant_and_1_year_exercise_qualifies() {
        let mut i = baseline();
        i.months_since_grant = 24;
        i.months_since_exercise = 12;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );
    }

    #[test]
    fn employment_requirement_failed() {
        let mut i = baseline();
        i.was_employee_throughout_grant_to_exercise_minus_3_months = false;
        i.employee_died_or_became_disabled = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::EmploymentRequirementFailed);
        assert_eq!(out.ordinary_compensation_income_cents, 40_00);
    }

    #[test]
    fn death_extension_satisfies_employment_requirement() {
        let mut i = baseline();
        i.was_employee_throughout_grant_to_exercise_minus_3_months = false;
        i.employee_died_or_became_disabled = true;
        let out = check(&i);
        // Death extension means employment requirement satisfied
        assert_eq!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );
    }

    #[test]
    fn espp_qualifying_disposition() {
        let mut i = baseline();
        i.option_type = OptionType::EmployeeStockPurchasePlanSection423;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );
    }

    #[test]
    fn espp_disqualifying_disposition() {
        let mut i = baseline();
        i.option_type = OptionType::EmployeeStockPurchasePlanSection423;
        i.months_since_grant = 12; // < 24
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421bDisqualifyingDispositionCompensation
        );
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 421(a)"));
        assert!(joined.contains("§ 421(a)(1)"));
        assert!(joined.contains("§ 421(a)(2)"));
        assert!(joined.contains("§ 421(a)(3)"));
        assert!(joined.contains("§ 421(b)"));
        assert!(joined.contains("§ 422"));
        assert!(joined.contains("§ 422(a)(1)"));
        assert!(joined.contains("§ 422(a)(2)"));
        assert!(joined.contains("§ 423"));
        assert!(joined.contains("§ 424(e)/(f)"));
        assert!(joined.contains("§ 56(b)(3)"));
        assert!(joined.contains("§ 162"));
        assert!(joined.contains("§ 83"));
        assert!(joined.contains("§ 1042"));
        assert!(joined.contains("§ 1222"));
        assert!(joined.contains("§ 6039"));
        assert!(joined.contains("Form 3921"));
        assert!(joined.contains("Form 3922"));
    }

    #[test]
    fn note_pins_general_rule_3_parts() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NO INCOME at time of exercise"));
        assert!(joined.contains("NO § 162 DEDUCTION"));
        assert!(joined.contains("only option price"));
    }

    #[test]
    fn note_pins_disqualifying_disposition_rule() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("DISQUALIFYING DISPOSITION"));
        assert!(joined.contains("INCREASE in FMV over option price"));
        assert!(joined.contains("COMPENSATION"));
    }

    #[test]
    fn note_pins_iso_2_1_holding_requirements() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("2 YEARS from GRANT"));
        assert!(joined.contains("1 YEAR from TRANSFER"));
        assert!(joined.contains("BOTH periods must be satisfied"));
    }

    #[test]
    fn note_pins_employment_requirement_3_months() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("3 MONTHS BEFORE"));
        assert!(joined.contains("DEATH extends"));
        assert!(joined.contains("DISABILITY extends"));
    }

    #[test]
    fn note_pins_amt_preference() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("AMT PREFERENCE"));
        assert!(joined.contains("phantom AMT income"));
    }

    #[test]
    fn note_pins_form_3921_reporting() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Form 3921"));
        assert!(joined.contains("Form 3922"));
        assert!(joined.contains("§ 6039"));
    }

    #[test]
    fn note_pins_1042_coordination() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 1042 (iter 480"));
        assert!(joined.contains("NOT 'qualified securities'"));
    }

    #[test]
    fn note_pins_83_coordination() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 83"));
        assert!(joined.contains("OVERRIDES § 83"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_422"));
        assert!(joined.contains("section_423"));
        assert!(joined.contains("section_409a"));
        assert!(joined.contains("section_83"));
        assert!(joined.contains("section_1042"));
        assert!(joined.contains("section_4978"));
    }

    #[test]
    fn truth_table_six_severity_cells() {
        // NQSO not applicable
        let c1 = check(&Input {
            option_type: OptionType::NonqualifiedStockOption,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // Grant
        let c2 = check(&Input {
            event_type: EventType::Grant,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::Section421aGrantNoIncome);

        // Exercise
        let c3 = check(&Input {
            event_type: EventType::Exercise,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::Section421aExerciseNoIncome);

        // Qualifying disposition
        let c4 = check(&baseline());
        assert_eq!(
            c4.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );

        // Disqualifying disposition
        let c5 = check(&Input {
            months_since_grant: 12,
            ..baseline()
        });
        assert_eq!(
            c5.severity,
            Severity::Section421bDisqualifyingDispositionCompensation
        );

        // Employment requirement failed
        let c6 = check(&Input {
            was_employee_throughout_grant_to_exercise_minus_3_months: false,
            employee_died_or_became_disabled: false,
            ..baseline()
        });
        assert_eq!(c6.severity, Severity::EmploymentRequirementFailed);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            option_exercise_price_cents: 0,
            fmv_at_exercise_cents: u64::MAX,
            fmv_at_disposition_cents: u64::MAX,
            ..baseline()
        };
        let out = check(&i);
        // No panic
        assert!(matches!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        ));
    }

    #[test]
    fn realistic_trader_iso_10000_shares_qualifying_disposition() {
        // Trader-employee exercises 10,000 ISO shares at $5 strike, FMV $50
        // at exercise = $450K AMT preference; holds 2+ years and sells at $100
        let mut i = baseline();
        i.option_exercise_price_cents = 5_00 * 10_000; // $50K exercise cost
        i.fmv_at_exercise_cents = 50_00 * 10_000; // $500K FMV at exercise
        i.fmv_at_disposition_cents = 100_00 * 10_000; // $1M FMV at disposition
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421aQualifyingDispositionCapitalGain
        );
        // Capital gain = $1M - $50K = $950K
        assert_eq!(out.capital_gain_cents, 95_00 * 10_000);
        // AMT preference at exercise = $500K - $50K = $450K
        assert_eq!(out.amt_preference_at_exercise_cents, 45_00 * 10_000);
    }

    #[test]
    fn realistic_trader_iso_premature_sale_disqualifying() {
        // Same 10,000 shares but sold within 1 year of exercise
        let mut i = baseline();
        i.option_exercise_price_cents = 5_00 * 10_000;
        i.fmv_at_exercise_cents = 50_00 * 10_000;
        i.fmv_at_disposition_cents = 100_00 * 10_000;
        i.months_since_exercise = 6;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::Section421bDisqualifyingDispositionCompensation
        );
        // Compensation = $500K - $50K = $450K ordinary income
        assert_eq!(out.ordinary_compensation_income_cents, 45_00 * 10_000);
        // Capital gain = $1M - $500K = $500K
        assert_eq!(out.capital_gain_cents, 50_00 * 10_000);
    }
}
