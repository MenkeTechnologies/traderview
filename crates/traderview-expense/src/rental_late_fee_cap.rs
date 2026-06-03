//! Late-fee cap + grace-period compliance framework for residential rentals.
//!
//! State law sharply constrains residential late-fee charges, both as to the maximum
//! amount and the minimum grace period before a fee may attach. Excessive late fees
//! are unenforceable under common-law liquidated-damages doctrine (must be a reasonable
//! estimate of actual damages, not a penalty) and create statutory exposure under
//! consumer-protection statutes (deceptive-practices, unconscionability).
//!
//! Caps and grace periods vary sharply:
//!
//! - NY RPL § 238-a (HSTPA 2019): late fee capped at the LESSER of $50 OR 5% of monthly
//!   rent; 5-day grace period required.
//! - WA RCW 59.18.170: NO statutory cap statewide, but landlord may not charge any late
//!   fee until rent is more than 5 days late.
//! - CO Rev. Stat. § 38-12-105 (HB 23-1099): late fee capped at the GREATER of $50 OR
//!   5% of monthly rent; 7-day grace period required.
//! - IL Chicago RLTO § 5-12-140(h): late fee capped at $10/month plus 5% per month for
//!   rent over $500.
//! - TX Prop. Code § 92.019: late fee must be "reasonable estimate" of damages.
//!   Statutory safe harbor: 10% for properties with 4 OR fewer units; 12% for
//!   properties with 5 OR more units. 2-day grace period required.
//! - CA Civ. Code § 1671: late fee must be reasonable estimate of damages (liquidated-
//!   damages doctrine); industry standard ~5% or ~$50. No statutory cap.
//! - MA Gen. L. ch. 186 § 15B(1)(c): no late fee for first 30 days after rent due
//!   (longest grace period of surveyed states).
//! - FL: NO statewide cap or grace period for residential (Fla. Stat. § 83.808 covers
//!   only mobile-home tenancies).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - landager.com/en/property-compliance/usa/new-york/late-fees
//! - rentlatefee.com/blog/understanding-rent-late-fees-state-guide
//! - hemlane.com/resources/late-fee-laws-by-state/
//! - washingtonlawhelp.org/en/some-cities-and-counties-have-stronger-protections-renters
//! - rentlatefee.com/calculator/new-york

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Washington,
    Colorado,
    IllinoisChicagoRlto,
    Texas4OrFewerUnits,
    Texas5OrMoreUnits,
    Massachusetts,
    Florida,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    LateFeeWithinCapAndGraceCompliant,
    LateFeeChargedBeforeGracePeriodExpired,
    LateFeeExceedsStatutoryCap,
    NoStatutoryCapCommonLawReasonablenessTest,
    NoLateFeeChargedCompliant,
    LongestGracePeriodMassachusettsThirtyDays,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub monthly_rent_cents: u64,
    pub late_fee_charged_cents: u64,
    pub days_after_rent_due_late_fee_charged: u32,
}

pub type RentalLateFeeCapInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_cap_cents: u64,
    pub statutory_grace_period_days: u32,
    pub excess_amount_cents: u64,
    pub note: String,
}

pub type RentalLateFeeCapOutput = Output;
pub type RentalLateFeeCapResult = Output;

const NY_FLAT_CAP_CENTS: u64 = 5_000;
const NY_PERCENT_CAP_BPS: u64 = 500;
const NY_GRACE_PERIOD_DAYS: u32 = 5;
const WA_GRACE_PERIOD_DAYS: u32 = 5;
const CO_FLAT_FLOOR_CENTS: u64 = 5_000;
const CO_PERCENT_FLOOR_BPS: u64 = 500;
const CO_GRACE_PERIOD_DAYS: u32 = 7;
const IL_CHICAGO_FLAT_CAP_CENTS: u64 = 1_000;
const IL_CHICAGO_PERCENT_CAP_BPS: u64 = 500;
const IL_CHICAGO_RENT_THRESHOLD_CENTS: u64 = 50_000;
const TX_4_OR_FEWER_UNITS_PERCENT_CAP_BPS: u64 = 1_000;
const TX_5_OR_MORE_UNITS_PERCENT_CAP_BPS: u64 = 1_200;
const TX_GRACE_PERIOD_DAYS: u32 = 2;
const MA_GRACE_PERIOD_DAYS: u32 = 30;
const DEFAULT_GRACE_PERIOD_DAYS: u32 = 5;

#[must_use]
pub fn check(input: &Input) -> Output {
    if input.late_fee_charged_cents == 0 {
        return Output {
            severity: Severity::NoLateFeeChargedCompliant,
            statutory_cap_cents: 0,
            statutory_grace_period_days: 0,
            excess_amount_cents: 0,
            note: "No late fee charged — compliant by definition. Note: in jurisdictions \
                   that require a written lease term to authorize ANY late fee (TX, IL, MA), \
                   the absence of a lease clause prohibits future charges as well."
                .to_string(),
        };
    }

    let (statutory_cap, grace_period, statutory_text) = compute_jurisdiction_limits(input);

    if input.days_after_rent_due_late_fee_charged < grace_period {
        return Output {
            severity: Severity::LateFeeChargedBeforeGracePeriodExpired,
            statutory_cap_cents: statutory_cap,
            statutory_grace_period_days: grace_period,
            excess_amount_cents: input.late_fee_charged_cents,
            note: format!(
                "Late fee charged at day {} after rent due, before the {}-day statutory \
                 grace period expired. {} Entire fee (${}) unenforceable; tenant may \
                 recover full fee + statutory damages where applicable + attorney fees.",
                input.days_after_rent_due_late_fee_charged,
                grace_period,
                statutory_text,
                input.late_fee_charged_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Massachusetts) {
        return Output {
            severity: Severity::LongestGracePeriodMassachusettsThirtyDays,
            statutory_cap_cents: 0,
            statutory_grace_period_days: MA_GRACE_PERIOD_DAYS,
            excess_amount_cents: 0,
            note: format!(
                "Massachusetts Gen. L. ch. 186 § 15B(1)(c) prohibits ANY late fee charge for \
                 the first 30 days after rent is due — longest grace period of surveyed \
                 states. Late fee charged at day {} compliant only because past the 30-day \
                 grace period; fee amount must still be a reasonable estimate of damages \
                 under common-law liquidated-damages doctrine.",
                input.days_after_rent_due_late_fee_charged
            ),
        };
    }

    if matches!(
        input.jurisdiction,
        Jurisdiction::Washington
            | Jurisdiction::California
            | Jurisdiction::Florida
            | Jurisdiction::Default
    ) {
        return Output {
            severity: Severity::NoStatutoryCapCommonLawReasonablenessTest,
            statutory_cap_cents: 0,
            statutory_grace_period_days: grace_period,
            excess_amount_cents: 0,
            note: format!(
                "{} Late fee (${}) charged at day {} after rent due passes the grace-period \
                 test. NO statutory dollar cap; common-law liquidated-damages doctrine \
                 requires the fee be a reasonable estimate of actual damages, not a penalty. \
                 Industry standard 5%-10% of monthly rent or $25-$75 typically considered \
                 reasonable; >10% of monthly rent triggers presumption of unenforceable \
                 penalty under CA Civ. Code § 1671 + parallel state doctrine.",
                statutory_text,
                input.late_fee_charged_cents / 100,
                input.days_after_rent_due_late_fee_charged
            ),
        };
    }

    if input.late_fee_charged_cents > statutory_cap {
        let excess = input.late_fee_charged_cents.saturating_sub(statutory_cap);
        return Output {
            severity: Severity::LateFeeExceedsStatutoryCap,
            statutory_cap_cents: statutory_cap,
            statutory_grace_period_days: grace_period,
            excess_amount_cents: excess,
            note: format!(
                "Late fee (${}) EXCEEDS statutory cap (${}). {} Excess (${}) unenforceable; \
                 tenant entitled to refund + statutory damages where applicable + attorney \
                 fees. Charging an unenforceable fee can also be deceptive practice under \
                 state UDAP statutes.",
                input.late_fee_charged_cents / 100,
                statutory_cap / 100,
                statutory_text,
                excess / 100
            ),
        };
    }

    Output {
        severity: Severity::LateFeeWithinCapAndGraceCompliant,
        statutory_cap_cents: statutory_cap,
        statutory_grace_period_days: grace_period,
        excess_amount_cents: 0,
        note: format!(
            "Compliant: late fee (${}) within statutory cap (${}) AND charged at day {} \
             after rent due (after {}-day grace period). {} Retain dated proof of rent-due \
             date and fee-charge date.",
            input.late_fee_charged_cents / 100,
            statutory_cap / 100,
            input.days_after_rent_due_late_fee_charged,
            grace_period,
            statutory_text
        ),
    }
}

fn compute_jurisdiction_limits(input: &Input) -> (u64, u32, String) {
    match input.jurisdiction {
        Jurisdiction::NewYork => {
            let percent_cap = input
                .monthly_rent_cents
                .saturating_mul(NY_PERCENT_CAP_BPS)
                .saturating_div(10_000);
            let cap = NY_FLAT_CAP_CENTS.min(percent_cap);
            (
                cap,
                NY_GRACE_PERIOD_DAYS,
                format!(
                    "NY RPL § 238-a (HSTPA 2019) caps late fee at LESSER of $50 OR 5% of \
                     monthly rent (${}); grace period {} days.",
                    cap / 100,
                    NY_GRACE_PERIOD_DAYS
                ),
            )
        }
        Jurisdiction::Colorado => {
            let percent_floor = input
                .monthly_rent_cents
                .saturating_mul(CO_PERCENT_FLOOR_BPS)
                .saturating_div(10_000);
            let cap = CO_FLAT_FLOOR_CENTS.max(percent_floor);
            (
                cap,
                CO_GRACE_PERIOD_DAYS,
                format!(
                    "CO Rev. Stat. § 38-12-105 (HB 23-1099) caps late fee at GREATER of $50 \
                     OR 5% of monthly rent (${}); grace period {} days.",
                    cap / 100,
                    CO_GRACE_PERIOD_DAYS
                ),
            )
        }
        Jurisdiction::IllinoisChicagoRlto => {
            let percent_addon = if input.monthly_rent_cents > IL_CHICAGO_RENT_THRESHOLD_CENTS {
                input
                    .monthly_rent_cents
                    .saturating_sub(IL_CHICAGO_RENT_THRESHOLD_CENTS)
                    .saturating_mul(IL_CHICAGO_PERCENT_CAP_BPS)
                    .saturating_div(10_000)
            } else {
                0
            };
            let cap = IL_CHICAGO_FLAT_CAP_CENTS.saturating_add(percent_addon);
            (
                cap,
                DEFAULT_GRACE_PERIOD_DAYS,
                format!(
                    "IL Chicago RLTO § 5-12-140(h) caps late fee at $10 + 5% per month for \
                     rent over $500 (${}).",
                    cap / 100
                ),
            )
        }
        Jurisdiction::Texas4OrFewerUnits => {
            let cap = input
                .monthly_rent_cents
                .saturating_mul(TX_4_OR_FEWER_UNITS_PERCENT_CAP_BPS)
                .saturating_div(10_000);
            (
                cap,
                TX_GRACE_PERIOD_DAYS,
                format!(
                    "TX Prop. Code § 92.019 statutory safe-harbor cap of 10% for properties \
                     with 4 or fewer units (${}); grace period {} days.",
                    cap / 100,
                    TX_GRACE_PERIOD_DAYS
                ),
            )
        }
        Jurisdiction::Texas5OrMoreUnits => {
            let cap = input
                .monthly_rent_cents
                .saturating_mul(TX_5_OR_MORE_UNITS_PERCENT_CAP_BPS)
                .saturating_div(10_000);
            (
                cap,
                TX_GRACE_PERIOD_DAYS,
                format!(
                    "TX Prop. Code § 92.019 statutory safe-harbor cap of 12% for properties \
                     with 5 or more units (${}); grace period {} days.",
                    cap / 100,
                    TX_GRACE_PERIOD_DAYS
                ),
            )
        }
        Jurisdiction::Massachusetts => (0, MA_GRACE_PERIOD_DAYS, String::new()),
        Jurisdiction::Washington => (
            0,
            WA_GRACE_PERIOD_DAYS,
            "WA RCW 59.18.170 requires landlord wait until rent is more than 5 days late \
             before charging any late fee; no statutory dollar cap statewide."
                .to_string(),
        ),
        Jurisdiction::California => (
            0,
            DEFAULT_GRACE_PERIOD_DAYS,
            "CA Civ. Code § 1671 requires late fee be reasonable estimate of damages; no \
             statutory dollar cap. Industry standard 5% or $50."
                .to_string(),
        ),
        Jurisdiction::Florida => (
            0,
            DEFAULT_GRACE_PERIOD_DAYS,
            "FL has NO statewide late-fee cap or grace period for residential tenancies \
             (Fla. Stat. § 83.808 covers only mobile-home tenancies)."
                .to_string(),
        ),
        Jurisdiction::Default => (
            0,
            DEFAULT_GRACE_PERIOD_DAYS,
            "Default jurisdiction: no specific statutory cap mapped; common-law \
             liquidated-damages doctrine requires fee be reasonable estimate of damages."
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ny() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYork,
            monthly_rent_cents: 3_000_00,
            late_fee_charged_cents: 5_000,
            days_after_rent_due_late_fee_charged: 6,
        }
    }

    #[test]
    fn no_late_fee_charged_compliant() {
        let mut input = base_ny();
        input.late_fee_charged_cents = 0;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoLateFeeChargedCompliant);
        assert_eq!(output.excess_amount_cents, 0);
    }

    #[test]
    fn new_york_50_dollar_late_fee_within_cap_compliant() {
        let input = base_ny();
        let output = check(&input);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
        assert_eq!(output.statutory_cap_cents, 5_000);
        assert_eq!(output.statutory_grace_period_days, 5);
        assert!(output.note.contains("RPL § 238-a"));
    }

    #[test]
    fn new_york_high_rent_5_percent_cap_caps_at_50_dollars() {
        let mut input = base_ny();
        input.monthly_rent_cents = 10_000_00;
        input.late_fee_charged_cents = 5_000;
        let output = check(&input);
        // 5% of $10K = $500 > $50 flat cap → cap = $50
        assert_eq!(output.statutory_cap_cents, 5_000);
    }

    #[test]
    fn new_york_low_rent_5_percent_cap_caps_below_50() {
        let mut input = base_ny();
        input.monthly_rent_cents = 500_00;
        input.late_fee_charged_cents = 2_500;
        let output = check(&input);
        // 5% of $500 = $25 < $50 flat → cap = $25
        assert_eq!(output.statutory_cap_cents, 2_500);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
    }

    #[test]
    fn new_york_75_dollar_late_fee_exceeds_cap() {
        let mut input = base_ny();
        input.late_fee_charged_cents = 7_500;
        let output = check(&input);
        assert_eq!(output.severity, Severity::LateFeeExceedsStatutoryCap);
        assert_eq!(output.excess_amount_cents, 2_500);
    }

    #[test]
    fn new_york_late_fee_before_grace_period_unenforceable() {
        let mut input = base_ny();
        input.days_after_rent_due_late_fee_charged = 4;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LateFeeChargedBeforeGracePeriodExpired
        );
        assert_eq!(output.excess_amount_cents, 5_000);
    }

    #[test]
    fn new_york_at_5_day_boundary_compliant() {
        let mut input = base_ny();
        input.days_after_rent_due_late_fee_charged = 5;
        let output = check(&input);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
    }

    #[test]
    fn colorado_greater_of_50_or_5_percent_cap() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Colorado;
        input.monthly_rent_cents = 10_000_00;
        input.late_fee_charged_cents = 50_000;
        input.days_after_rent_due_late_fee_charged = 8;
        let output = check(&input);
        // 5% of $10K = $500 > $50 → cap = $500
        assert_eq!(output.statutory_cap_cents, 50_000);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
        assert!(output.note.contains("HB 23-1099"));
    }

    #[test]
    fn colorado_7_day_grace_period_violated_at_6_days() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Colorado;
        input.days_after_rent_due_late_fee_charged = 6;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LateFeeChargedBeforeGracePeriodExpired
        );
        assert_eq!(output.statutory_grace_period_days, 7);
    }

    #[test]
    fn illinois_chicago_rlto_10_plus_5_pct_formula() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.monthly_rent_cents = 2_000_00;
        input.late_fee_charged_cents = 8_500;
        let output = check(&input);
        // $10 + 5% of ($2,000 - $500) = $10 + $75 = $85
        assert_eq!(output.statutory_cap_cents, 8_500);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
    }

    #[test]
    fn illinois_chicago_low_rent_under_500_only_flat_10() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.monthly_rent_cents = 400_00;
        input.late_fee_charged_cents = 1_000;
        let output = check(&input);
        assert_eq!(output.statutory_cap_cents, 1_000);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
    }

    #[test]
    fn texas_4_or_fewer_units_10_pct_cap() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Texas4OrFewerUnits;
        input.monthly_rent_cents = 2_000_00;
        input.late_fee_charged_cents = 20_000;
        input.days_after_rent_due_late_fee_charged = 3;
        let output = check(&input);
        // 10% of $2,000 = $200
        assert_eq!(output.statutory_cap_cents, 20_000);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
        assert!(output.note.contains("§ 92.019"));
    }

    #[test]
    fn texas_5_or_more_units_12_pct_cap() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Texas5OrMoreUnits;
        input.monthly_rent_cents = 2_000_00;
        input.late_fee_charged_cents = 24_000;
        input.days_after_rent_due_late_fee_charged = 3;
        let output = check(&input);
        // 12% of $2,000 = $240
        assert_eq!(output.statutory_cap_cents, 24_000);
        assert_eq!(output.severity, Severity::LateFeeWithinCapAndGraceCompliant);
    }

    #[test]
    fn texas_2_day_grace_period_violated_at_1_day() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Texas4OrFewerUnits;
        input.days_after_rent_due_late_fee_charged = 1;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LateFeeChargedBeforeGracePeriodExpired
        );
        assert_eq!(output.statutory_grace_period_days, 2);
    }

    #[test]
    fn washington_no_statutory_cap_uses_common_law_test() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoStatutoryCapCommonLawReasonablenessTest
        );
        assert!(output.note.contains("RCW 59.18.170"));
        assert!(output.note.contains("liquidated-damages"));
    }

    #[test]
    fn washington_grace_period_5_days_violated_at_4_days() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Washington;
        input.days_after_rent_due_late_fee_charged = 4;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LateFeeChargedBeforeGracePeriodExpired
        );
        assert_eq!(output.statutory_grace_period_days, 5);
    }

    #[test]
    fn california_no_cap_common_law_reasonableness() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::California;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoStatutoryCapCommonLawReasonablenessTest
        );
        assert!(output.note.contains("§ 1671"));
    }

    #[test]
    fn florida_no_statewide_cap_or_grace_period() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoStatutoryCapCommonLawReasonablenessTest
        );
        assert!(output.note.contains("§ 83.808"));
    }

    #[test]
    fn massachusetts_30_day_grace_period_longest_of_surveyed_states() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Massachusetts;
        input.days_after_rent_due_late_fee_charged = 31;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LongestGracePeriodMassachusettsThirtyDays
        );
        assert_eq!(output.statutory_grace_period_days, 30);
        assert!(output.note.contains("§ 15B(1)(c)"));
    }

    #[test]
    fn massachusetts_within_30_day_grace_period_unenforceable() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Massachusetts;
        input.days_after_rent_due_late_fee_charged = 29;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LateFeeChargedBeforeGracePeriodExpired
        );
        assert_eq!(output.statutory_grace_period_days, 30);
    }

    #[test]
    fn default_jurisdiction_no_cap_common_law() {
        let mut input = base_ny();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoStatutoryCapCommonLawReasonablenessTest
        );
    }

    #[test]
    fn ny_flat_cap_constant_pins_50_dollars() {
        assert_eq!(NY_FLAT_CAP_CENTS, 5_000);
    }

    #[test]
    fn ny_percent_cap_constant_pins_5_pct() {
        assert_eq!(NY_PERCENT_CAP_BPS, 500);
    }

    #[test]
    fn ny_grace_period_constant_pins_5_days() {
        assert_eq!(NY_GRACE_PERIOD_DAYS, 5);
    }

    #[test]
    fn wa_grace_period_constant_pins_5_days() {
        assert_eq!(WA_GRACE_PERIOD_DAYS, 5);
    }

    #[test]
    fn co_flat_floor_constant_pins_50_dollars() {
        assert_eq!(CO_FLAT_FLOOR_CENTS, 5_000);
    }

    #[test]
    fn co_grace_period_constant_pins_7_days() {
        assert_eq!(CO_GRACE_PERIOD_DAYS, 7);
    }

    #[test]
    fn il_chicago_flat_cap_constant_pins_10_dollars() {
        assert_eq!(IL_CHICAGO_FLAT_CAP_CENTS, 1_000);
    }

    #[test]
    fn il_chicago_rent_threshold_constant_pins_500() {
        assert_eq!(IL_CHICAGO_RENT_THRESHOLD_CENTS, 50_000);
    }

    #[test]
    fn tx_4_or_fewer_units_percent_constant_pins_10_pct() {
        assert_eq!(TX_4_OR_FEWER_UNITS_PERCENT_CAP_BPS, 1_000);
    }

    #[test]
    fn tx_5_or_more_units_percent_constant_pins_12_pct() {
        assert_eq!(TX_5_OR_MORE_UNITS_PERCENT_CAP_BPS, 1_200);
    }

    #[test]
    fn tx_grace_period_constant_pins_2_days() {
        assert_eq!(TX_GRACE_PERIOD_DAYS, 2);
    }

    #[test]
    fn ma_grace_period_constant_pins_30_days() {
        assert_eq!(MA_GRACE_PERIOD_DAYS, 30);
    }

    #[test]
    fn very_large_rent_no_overflow_in_percent_calc() {
        let mut input = base_ny();
        input.monthly_rent_cents = u64::MAX / 10;
        let output = check(&input);
        // saturating_mul defense
        assert!(output.statutory_cap_cents > 0);
    }

    #[test]
    fn zero_rent_no_panic() {
        let mut input = base_ny();
        input.monthly_rent_cents = 0;
        input.late_fee_charged_cents = 100;
        let output = check(&input);
        // NY: lesser of $50 or 5% of $0 = $0 → cap = $0
        assert_eq!(output.statutory_cap_cents, 0);
        assert_eq!(output.severity, Severity::LateFeeExceedsStatutoryCap);
    }
}
