//! IRC § 737 recognition of precontribution gain in case of certain
//! distributions to a contributing partner.
//!
//! One half of the Subchapter K "anti-mixing-bowl" anti-abuse regime:
//! § 737 is the **contributing-partner-side** recognition rule
//! triggered when a partner who contributed appreciated § 704(c)
//! property receives a distribution of OTHER property (not money)
//! within 7 years of the original contribution. The other half is
//! **§ 704(c)(1)(B)** — the noncontributing-partner-side
//! recognition rule triggered when the contributed property itself
//! is distributed to a different partner within 7 years.
//!
//! Together § 737 and § 704(c)(1)(B) prevent tax-free property swaps
//! disguised as partnership contributions and distributions (the
//! "mixing bowl" — putting appreciated property in, taking different
//! appreciated property out, both partners walking away with
//! economic gain without recognition).
//!
//! Statutory chain:
//!
//! - Originally enacted by Energy Policy Act of 1992 (Pub. L. 102-486)
//!   with a 5-year recognition window.
//! - Extended to 7 years by Taxpayer Relief Act of 1997 (Pub. L.
//!   105-34).
//!
//! **§ 737(a) recognition rule**: contributing partner recognizes
//! gain on a distribution of property OTHER THAN MONEY equal to the
//! LESSER of:
//!
//! 1. the **net precontribution gain** (§ 737(b)) — the § 704(c)(1)(B)
//!    gain that would be recognized if all property held by the
//!    partnership immediately before the distribution that was
//!    contributed by the distributee partner WITHIN 7 YEARS were
//!    distributed to another partner; or
//! 2. the **excess** of the FMV of the distributed property over the
//!    adjusted basis of the partner's interest in the partnership
//!    (the so-called "excess distribution amount").
//!
//! Money distributions are NOT subject to § 737 — they fall under
//! § 731(a)(1) gain-on-cash-distribution rules instead.
//!
//! § 737(c) provides matching basis adjustments to (1) the partner's
//! interest in the partnership and (2) the contributed property held
//! by the partnership, to prevent double-counting on subsequent
//! disposition.
//!
//! Coordination with § 751 (hot assets) per Treas. Reg. § 1.737-2:
//! § 751 character applies FIRST, then § 737 layers on top of the
//! remaining capital-character portion.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const SECTION_737_LOOKBACK_PERIOD_YEARS: u32 = 7;
#[allow(dead_code)]
pub const SECTION_737_LOOKBACK_PERIOD_MONTHS: u32 = 84;
#[allow(dead_code)]
pub const ENERGY_POLICY_ACT_1992_ENACTMENT_YEAR: u32 = 1992;
#[allow(dead_code)]
pub const TRA_1997_EXTENSION_TO_7_YEARS_YEAR: u32 = 1997;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LesserOfBranch {
    NotApplicable,
    NetPrecontributionGainIsLesser,
    ExcessFmvOverOutsideBasisIsLesser,
    BothAreZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoRecognitionDistributionOutside7YearWindow,
    NoRecognitionDistributionIsMoneyOnly,
    NoRecognitionNoNetPrecontributionGain,
    NoRecognitionExcessFmvOverOutsideBasisIsZero,
    RecognitionGainNetPrecontributionLesserOfBranch,
    RecognitionGainExcessFmvOverOutsideBasisLesserOfBranch,
    RecognitionGainBranchesEqual,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub months_since_contribution: u32,
    pub distribution_is_money_only: bool,
    pub net_precontribution_gain_cents: u64,
    pub distributed_property_fmv_cents: u64,
    pub distributee_outside_basis_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub recognition_triggered: bool,
    pub gain_recognized_cents: u64,
    pub net_precontribution_gain_cents: u64,
    pub excess_fmv_over_outside_basis_cents: u64,
    pub lesser_of_branch: LesserOfBranch,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section737Input = Input;
pub type Section737Output = Output;
pub type Section737Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 737(a) (recognition of gain on distribution to contributing partner)".to_string(),
        "IRC § 737(a)(1) (gain = lesser of net precontribution gain or excess distribution)".to_string(),
        "IRC § 737(b) (net precontribution gain definition; 7-year period)".to_string(),
        "IRC § 737(c) (basis adjustments)".to_string(),
        "IRC § 737(d) (exceptions, including § 751 coordination)".to_string(),
        "IRC § 704(c)(1)(B) (noncontributing-partner-side mixing-bowl rule)".to_string(),
        "IRC § 731(a)(1) (money distribution gain recognition — § 737 carves out money)".to_string(),
        "Treas. Reg. § 1.737-1 (general rules)".to_string(),
        "Treas. Reg. § 1.737-2 (exceptions and special rules)".to_string(),
        "Treas. Reg. § 1.737-3 (basis adjustments and recovery)".to_string(),
        "Energy Policy Act of 1992 (Pub. L. 102-486) — original enactment with 5-year window".to_string(),
        "Taxpayer Relief Act of 1997 (Pub. L. 105-34) — extended to 7-year window".to_string(),
    ];

    if input.months_since_contribution > SECTION_737_LOOKBACK_PERIOD_MONTHS {
        notes.push(format!(
            "Distribution {} months after contribution exceeds {}-month (7-year) § 737(b) window — no recognition.",
            input.months_since_contribution,
            SECTION_737_LOOKBACK_PERIOD_MONTHS
        ));
        return Output {
            severity: Severity::NoRecognitionDistributionOutside7YearWindow,
            recognition_triggered: false,
            gain_recognized_cents: 0,
            net_precontribution_gain_cents: input.net_precontribution_gain_cents,
            excess_fmv_over_outside_basis_cents: 0,
            lesser_of_branch: LesserOfBranch::NotApplicable,
            notes,
            citations,
        };
    }

    if input.distribution_is_money_only {
        notes.push("Money-only distribution — § 737 does not apply; § 731(a)(1) money-distribution rules govern.".to_string());
        return Output {
            severity: Severity::NoRecognitionDistributionIsMoneyOnly,
            recognition_triggered: false,
            gain_recognized_cents: 0,
            net_precontribution_gain_cents: input.net_precontribution_gain_cents,
            excess_fmv_over_outside_basis_cents: 0,
            lesser_of_branch: LesserOfBranch::NotApplicable,
            notes,
            citations,
        };
    }

    let excess_fmv = input
        .distributed_property_fmv_cents
        .saturating_sub(input.distributee_outside_basis_cents);
    let net_precontrib = input.net_precontribution_gain_cents;

    if net_precontrib == 0 {
        notes.push("No net precontribution gain — first prong of § 737(a) lesser-of test is zero; no recognition.".to_string());
        return Output {
            severity: Severity::NoRecognitionNoNetPrecontributionGain,
            recognition_triggered: false,
            gain_recognized_cents: 0,
            net_precontribution_gain_cents: 0,
            excess_fmv_over_outside_basis_cents: excess_fmv,
            lesser_of_branch: LesserOfBranch::BothAreZero,
            notes,
            citations,
        };
    }

    if excess_fmv == 0 {
        notes.push("Excess distribution amount is zero (FMV ≤ outside basis) — second prong of § 737(a) lesser-of test is zero; no recognition.".to_string());
        return Output {
            severity: Severity::NoRecognitionExcessFmvOverOutsideBasisIsZero,
            recognition_triggered: false,
            gain_recognized_cents: 0,
            net_precontribution_gain_cents: net_precontrib,
            excess_fmv_over_outside_basis_cents: 0,
            lesser_of_branch: LesserOfBranch::BothAreZero,
            notes,
            citations,
        };
    }

    let gain = net_precontrib.min(excess_fmv);
    let (severity, branch) = if net_precontrib < excess_fmv {
        notes.push(format!(
            "§ 737(a) recognition: net precontribution gain ${} < excess FMV over outside basis ${} = ${} recognized (net precontribution is lesser).",
            net_precontrib / 100,
            excess_fmv / 100,
            gain / 100
        ));
        (
            Severity::RecognitionGainNetPrecontributionLesserOfBranch,
            LesserOfBranch::NetPrecontributionGainIsLesser,
        )
    } else if excess_fmv < net_precontrib {
        notes.push(format!(
            "§ 737(a) recognition: excess FMV over outside basis ${} < net precontribution gain ${} = ${} recognized (excess distribution is lesser).",
            excess_fmv / 100,
            net_precontrib / 100,
            gain / 100
        ));
        (
            Severity::RecognitionGainExcessFmvOverOutsideBasisLesserOfBranch,
            LesserOfBranch::ExcessFmvOverOutsideBasisIsLesser,
        )
    } else {
        notes.push(format!(
            "§ 737(a) recognition: net precontribution gain equals excess FMV at ${} — both branches recognize the same amount.",
            gain / 100
        ));
        (
            Severity::RecognitionGainBranchesEqual,
            LesserOfBranch::NetPrecontributionGainIsLesser,
        )
    };

    Output {
        severity,
        recognition_triggered: true,
        gain_recognized_cents: gain,
        net_precontribution_gain_cents: net_precontrib,
        excess_fmv_over_outside_basis_cents: excess_fmv,
        lesser_of_branch: branch,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_recognition() -> Input {
        Input {
            months_since_contribution: 24,
            distribution_is_money_only: false,
            net_precontribution_gain_cents: 5_000_000,
            distributed_property_fmv_cents: 10_000_000,
            distributee_outside_basis_cents: 3_000_000,
        }
    }

    #[test]
    fn within_7_year_window_net_precontrib_lesser_recognized() {
        let out = check(&base_recognition());
        assert_eq!(
            out.severity,
            Severity::RecognitionGainNetPrecontributionLesserOfBranch
        );
        assert_eq!(out.gain_recognized_cents, 5_000_000);
        assert!(out.recognition_triggered);
        assert_eq!(
            out.lesser_of_branch,
            LesserOfBranch::NetPrecontributionGainIsLesser
        );
    }

    #[test]
    fn excess_fmv_lesser_than_net_precontrib_recognized() {
        let mut i = base_recognition();
        i.distributed_property_fmv_cents = 5_000_000;
        i.distributee_outside_basis_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::RecognitionGainExcessFmvOverOutsideBasisLesserOfBranch
        );
        assert_eq!(out.gain_recognized_cents, 2_000_000);
        assert_eq!(
            out.lesser_of_branch,
            LesserOfBranch::ExcessFmvOverOutsideBasisIsLesser
        );
    }

    #[test]
    fn distribution_outside_7_year_window_no_recognition() {
        let mut i = base_recognition();
        i.months_since_contribution = 85;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NoRecognitionDistributionOutside7YearWindow
        );
        assert!(!out.recognition_triggered);
    }

    #[test]
    fn distribution_exactly_at_84_months_still_in_window() {
        let mut i = base_recognition();
        i.months_since_contribution = 84;
        let out = check(&i);
        assert!(out.recognition_triggered);
    }

    #[test]
    fn distribution_at_85_months_outside_window() {
        let mut i = base_recognition();
        i.months_since_contribution = 85;
        let out = check(&i);
        assert!(!out.recognition_triggered);
    }

    #[test]
    fn money_only_distribution_no_recognition() {
        let mut i = base_recognition();
        i.distribution_is_money_only = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NoRecognitionDistributionIsMoneyOnly);
        assert!(!out.recognition_triggered);
    }

    #[test]
    fn no_net_precontribution_gain_no_recognition() {
        let mut i = base_recognition();
        i.net_precontribution_gain_cents = 0;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NoRecognitionNoNetPrecontributionGain
        );
        assert!(!out.recognition_triggered);
    }

    #[test]
    fn excess_fmv_zero_when_fmv_below_outside_basis_no_recognition() {
        let mut i = base_recognition();
        i.distributed_property_fmv_cents = 1_000_000;
        i.distributee_outside_basis_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NoRecognitionExcessFmvOverOutsideBasisIsZero
        );
        assert!(!out.recognition_triggered);
    }

    #[test]
    fn branches_equal_pick_either_record_correct_amount() {
        let mut i = base_recognition();
        i.net_precontribution_gain_cents = 4_000_000;
        i.distributed_property_fmv_cents = 7_000_000;
        i.distributee_outside_basis_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::RecognitionGainBranchesEqual);
        assert_eq!(out.gain_recognized_cents, 4_000_000);
    }

    #[test]
    fn citations_pin_737a_b_c_d() {
        let out = check(&base_recognition());
        assert!(out.citations.iter().any(|c| c.contains("§ 737(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 737(a)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 737(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 737(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 737(d)")));
    }

    #[test]
    fn citations_pin_704c1b_731a1_751_coordination() {
        let out = check(&base_recognition());
        assert!(out.citations.iter().any(|c| c.contains("§ 704(c)(1)(B)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 731(a)(1)")));
    }

    #[test]
    fn citations_pin_energy_policy_act_1992_and_tra_1997() {
        let out = check(&base_recognition());
        assert!(out.citations.iter().any(|c| c.contains("Energy Policy Act of 1992")));
        assert!(out.citations.iter().any(|c| c.contains("Taxpayer Relief Act of 1997")));
    }

    #[test]
    fn citations_pin_treas_reg_1_737_1_2_3() {
        let out = check(&base_recognition());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.737-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.737-2")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.737-3")));
    }

    #[test]
    fn constant_pin_7_year_lookback() {
        assert_eq!(SECTION_737_LOOKBACK_PERIOD_YEARS, 7);
    }

    #[test]
    fn constant_pin_84_month_lookback() {
        assert_eq!(SECTION_737_LOOKBACK_PERIOD_MONTHS, 84);
    }

    #[test]
    fn constant_pin_energy_policy_act_1992_year() {
        assert_eq!(ENERGY_POLICY_ACT_1992_ENACTMENT_YEAR, 1992);
    }

    #[test]
    fn constant_pin_tra_1997_extension_year() {
        assert_eq!(TRA_1997_EXTENSION_TO_7_YEARS_YEAR, 1997);
    }

    #[test]
    fn very_large_net_precontrib_saturating_no_overflow() {
        let mut i = base_recognition();
        i.net_precontribution_gain_cents = u64::MAX;
        i.distributed_property_fmv_cents = u64::MAX;
        i.distributee_outside_basis_cents = 0;
        let out = check(&i);
        assert_eq!(out.gain_recognized_cents, u64::MAX);
    }

    #[test]
    fn fmv_equals_outside_basis_no_excess_no_recognition() {
        let mut i = base_recognition();
        i.distributed_property_fmv_cents = 5_000_000;
        i.distributee_outside_basis_cents = 5_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NoRecognitionExcessFmvOverOutsideBasisIsZero
        );
    }
}
