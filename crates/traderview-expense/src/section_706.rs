//! IRC § 706 taxable years of partner and partnership.
//!
//! Three operative subsections governing partnership-tax timing:
//!
//! **§ 706(a)** — partner takes the partnership's year-end for
//! purposes of including the distributive share. If partnership has
//! Dec 31 year-end and partner has June 30 year-end, distributive
//! share is reported in partner's year that INCLUDES the
//! partnership's year-end.
//!
//! **§ 706(b) partnership-taxable-year selection hierarchy** —
//! three-tier test applied in strict order:
//!
//! 1. **Majority Interest Test** (§ 706(b)(1)(B)(i)): partners
//!    holding more than 50% of profits AND capital interests must
//!    have the same taxable year; partnership adopts that year.
//! 2. **Principal Partner Test** (§ 706(b)(1)(B)(ii)): if majority
//!    test fails, partnership uses the taxable year of all PRINCIPAL
//!    PARTNERS (each holding 5% or more of profits or capital). Only
//!    applies if all principal partners share the same year.
//! 3. **Least Aggregate Deferral Test** (§ 706(b)(1)(B)(iii)): final
//!    fallback — partnership uses the year that produces the LEAST
//!    aggregate deferral of income across all partners. **De minimis
//!    rule**: if the candidate year's aggregate deferral is less than
//!    0.5 (half-month deferral) different from the partnership's
//!    current year, current year is retained.
//!
//! **§ 706(c) closing of partnership taxable year**:
//!
//! - **§ 706(c)(1) general rule**: partnership year does NOT close
//!   on the death of a partner, entry of a new partner, partial
//!   liquidation of a partner's interest, or sale/exchange of a
//!   partner's PARTIAL interest.
//! - **§ 706(c)(2) exception**: partnership year DOES close with
//!   respect to a partner whose ENTIRE interest terminates (death,
//!   complete liquidation, complete sale or exchange). The partner
//!   takes a short-year distributive share through the termination
//!   date.
//!
//! **§ 706(d) varying-interest allocations**: when a partner's
//! interest changes during the taxable year, distributive share is
//! determined by one of two methods:
//!
//! - **Interim Closing Method** (default): partnership "closes the
//!   books" as of the change date; allocations to changing partner
//!   are determined as if a short partnership year ended on that
//!   date.
//! - **Proration Method**: proportional allocation based on time
//!   ownership. Available ONLY if elected in writing by the
//!   partners under Treas. Reg. § 1.706-4(f).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const MAJORITY_INTEREST_TEST_THRESHOLD_PERCENT: u32 = 50;
#[allow(dead_code)]
pub const PRINCIPAL_PARTNER_TEST_THRESHOLD_PERCENT: u32 = 5;
#[allow(dead_code)]
pub const DE_MINIMIS_LEAST_AGGREGATE_DEFERRAL_THRESHOLD_X_10: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    NotApplicable,
    PartnershipYearDetermination,
    PartnerTerminationEvent,
    VaryingInterestAllocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaryingInterestMethod {
    NotApplicable,
    InterimClosingMethodDefault,
    ProrationMethodIfElected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    PartnershipYearMajorityInterestTestApplies,
    PartnershipYearPrincipalPartnerTestApplies,
    PartnershipYearLeastAggregateDeferralTestApplies,
    PartnershipYearDeMinimisCurrentYearKept,
    EntireInterestTerminationPartnershipYearClosesForPartner,
    PartialInterestChangePartnershipYearDoesNotClose,
    InterimClosingMethodAppliedForVaryingInterests,
    ProrationMethodAppliedWithWrittenAgreement,
    ViolationProrationMethodWithoutWrittenAgreement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub analysis_type: AnalysisType,
    pub majority_interest_test_satisfied: bool,
    pub principal_partner_test_satisfied: bool,
    pub least_aggregate_deferral_change_months_x_10: u32,
    pub partner_entire_interest_terminated: bool,
    pub partner_partial_interest_changed: bool,
    pub varying_interest_method: VaryingInterestMethod,
    pub proration_method_written_partner_agreement: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub partnership_year_closes_for_partner: bool,
    pub distributive_share_method: VaryingInterestMethod,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section706Input = Input;
pub type Section706Output = Output;
pub type Section706Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 706(a) (partner includes distributive share in year containing partnership year-end)".to_string(),
        "IRC § 706(b)(1)(B)(i) (majority interest test — 50% threshold)".to_string(),
        "IRC § 706(b)(1)(B)(ii) (principal partner test — 5% threshold)".to_string(),
        "IRC § 706(b)(1)(B)(iii) (least aggregate deferral test)".to_string(),
        "IRC § 706(c)(1) (partnership year does not close on partial interest change)".to_string(),
        "IRC § 706(c)(2) (partnership year closes with respect to partner with entire interest termination)".to_string(),
        "IRC § 706(d) (varying-interest allocations)".to_string(),
        "Treas. Reg. § 1.706-1 (taxable years of partner and partnership)".to_string(),
        "Treas. Reg. § 1.706-4 (varying interests — interim closing vs. proration)".to_string(),
        "Treas. Reg. § 1.706-4(f) (proration-method written-agreement election)".to_string(),
    ];

    match input.analysis_type {
        AnalysisType::NotApplicable => {
            notes.push("No § 706 analysis triggered.".to_string());
            Output {
                severity: Severity::NotApplicable,
                partnership_year_closes_for_partner: false,
                distributive_share_method: VaryingInterestMethod::NotApplicable,
                notes,
                citations,
            }
        }
        AnalysisType::PartnershipYearDetermination => {
            if input.majority_interest_test_satisfied {
                notes.push(format!(
                    "§ 706(b)(1)(B)(i) Majority Interest Test satisfied: partners holding > {}% have same taxable year; partnership adopts that year.",
                    MAJORITY_INTEREST_TEST_THRESHOLD_PERCENT
                ));
                return Output {
                    severity: Severity::PartnershipYearMajorityInterestTestApplies,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                };
            }
            if input.principal_partner_test_satisfied {
                notes.push(format!(
                    "§ 706(b)(1)(B)(ii) Principal Partner Test: majority interest test failed; partnership adopts year of all partners holding ≥ {}% of profits or capital.",
                    PRINCIPAL_PARTNER_TEST_THRESHOLD_PERCENT
                ));
                return Output {
                    severity: Severity::PartnershipYearPrincipalPartnerTestApplies,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                };
            }
            if input.least_aggregate_deferral_change_months_x_10
                < DE_MINIMIS_LEAST_AGGREGATE_DEFERRAL_THRESHOLD_X_10
            {
                notes.push(format!(
                    "§ 706(b)(1)(B)(iii) de minimis exception: candidate year's deferral change {} (× 10) < {} threshold; current year retained.",
                    input.least_aggregate_deferral_change_months_x_10,
                    DE_MINIMIS_LEAST_AGGREGATE_DEFERRAL_THRESHOLD_X_10
                ));
                return Output {
                    severity: Severity::PartnershipYearDeMinimisCurrentYearKept,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                };
            }
            notes.push("§ 706(b)(1)(B)(iii) Least Aggregate Deferral Test: applies as fallback when neither majority nor principal partner tests yield a result.".to_string());
            Output {
                severity: Severity::PartnershipYearLeastAggregateDeferralTestApplies,
                partnership_year_closes_for_partner: false,
                distributive_share_method: VaryingInterestMethod::NotApplicable,
                notes,
                citations,
            }
        }
        AnalysisType::PartnerTerminationEvent => {
            if input.partner_entire_interest_terminated {
                notes.push("§ 706(c)(2): partner's entire interest terminated (death, complete liquidation, or complete sale/exchange); partnership year CLOSES with respect to this partner.".to_string());
                return Output {
                    severity: Severity::EntireInterestTerminationPartnershipYearClosesForPartner,
                    partnership_year_closes_for_partner: true,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                };
            }
            if input.partner_partial_interest_changed {
                notes.push("§ 706(c)(1): partner's PARTIAL interest changed (entry, partial liquidation, partial sale); partnership year does NOT close.".to_string());
                return Output {
                    severity: Severity::PartialInterestChangePartnershipYearDoesNotClose,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                };
            }
            notes.push("No partner termination or interest change event.".to_string());
            Output {
                severity: Severity::PartialInterestChangePartnershipYearDoesNotClose,
                partnership_year_closes_for_partner: false,
                distributive_share_method: VaryingInterestMethod::NotApplicable,
                notes,
                citations,
            }
        }
        AnalysisType::VaryingInterestAllocation => match input.varying_interest_method {
            VaryingInterestMethod::ProrationMethodIfElected => {
                if !input.proration_method_written_partner_agreement {
                    notes.push("§ 706(d) + Treas. Reg. § 1.706-4(f): proration method used without written partner agreement — per se procedural violation; default interim-closing method should have been applied.".to_string());
                    return Output {
                        severity: Severity::ViolationProrationMethodWithoutWrittenAgreement,
                        partnership_year_closes_for_partner: false,
                        distributive_share_method: VaryingInterestMethod::InterimClosingMethodDefault,
                        notes,
                        citations,
                    };
                }
                notes.push("§ 706(d): proration method applied with written partner agreement; allocations are time-proportional across the taxable year.".to_string());
                Output {
                    severity: Severity::ProrationMethodAppliedWithWrittenAgreement,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::ProrationMethodIfElected,
                    notes,
                    citations,
                }
            }
            VaryingInterestMethod::InterimClosingMethodDefault => {
                notes.push("§ 706(d): interim closing method (default) applied — partnership closes books as of partner-change date; short-year allocations to changing partner.".to_string());
                Output {
                    severity: Severity::InterimClosingMethodAppliedForVaryingInterests,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::InterimClosingMethodDefault,
                    notes,
                    citations,
                }
            }
            VaryingInterestMethod::NotApplicable => {
                notes.push("No varying-interest method recorded.".to_string());
                Output {
                    severity: Severity::NotApplicable,
                    partnership_year_closes_for_partner: false,
                    distributive_share_method: VaryingInterestMethod::NotApplicable,
                    notes,
                    citations,
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_pship_year() -> Input {
        Input {
            analysis_type: AnalysisType::PartnershipYearDetermination,
            majority_interest_test_satisfied: true,
            principal_partner_test_satisfied: false,
            least_aggregate_deferral_change_months_x_10: 0,
            partner_entire_interest_terminated: false,
            partner_partial_interest_changed: false,
            varying_interest_method: VaryingInterestMethod::NotApplicable,
            proration_method_written_partner_agreement: false,
        }
    }

    #[test]
    fn majority_interest_test_satisfied_first_tier_applies() {
        let out = check(&base_pship_year());
        assert_eq!(
            out.severity,
            Severity::PartnershipYearMajorityInterestTestApplies
        );
    }

    #[test]
    fn principal_partner_test_applies_when_majority_fails() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = false;
        i.principal_partner_test_satisfied = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearPrincipalPartnerTestApplies
        );
    }

    #[test]
    fn least_aggregate_deferral_test_applies_when_both_first_tiers_fail() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = false;
        i.principal_partner_test_satisfied = false;
        i.least_aggregate_deferral_change_months_x_10 = 20;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearLeastAggregateDeferralTestApplies
        );
    }

    #[test]
    fn de_minimis_keeps_current_year_when_below_half() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = false;
        i.principal_partner_test_satisfied = false;
        i.least_aggregate_deferral_change_months_x_10 = 4;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearDeMinimisCurrentYearKept
        );
    }

    #[test]
    fn de_minimis_at_exactly_5_steps_to_least_aggregate_deferral() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = false;
        i.principal_partner_test_satisfied = false;
        i.least_aggregate_deferral_change_months_x_10 = 5;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearLeastAggregateDeferralTestApplies
        );
    }

    #[test]
    fn entire_interest_termination_closes_partnership_year_for_partner() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::PartnerTerminationEvent;
        i.partner_entire_interest_terminated = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::EntireInterestTerminationPartnershipYearClosesForPartner
        );
        assert!(out.partnership_year_closes_for_partner);
    }

    #[test]
    fn partial_interest_change_does_not_close_partnership_year() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::PartnerTerminationEvent;
        i.partner_partial_interest_changed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartialInterestChangePartnershipYearDoesNotClose
        );
        assert!(!out.partnership_year_closes_for_partner);
    }

    #[test]
    fn interim_closing_method_default_applied() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::VaryingInterestAllocation;
        i.varying_interest_method = VaryingInterestMethod::InterimClosingMethodDefault;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::InterimClosingMethodAppliedForVaryingInterests
        );
    }

    #[test]
    fn proration_method_with_written_agreement_applied() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::VaryingInterestAllocation;
        i.varying_interest_method = VaryingInterestMethod::ProrationMethodIfElected;
        i.proration_method_written_partner_agreement = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ProrationMethodAppliedWithWrittenAgreement
        );
    }

    #[test]
    fn proration_method_without_written_agreement_violation() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::VaryingInterestAllocation;
        i.varying_interest_method = VaryingInterestMethod::ProrationMethodIfElected;
        i.proration_method_written_partner_agreement = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationProrationMethodWithoutWrittenAgreement
        );
        assert_eq!(
            out.distributive_share_method,
            VaryingInterestMethod::InterimClosingMethodDefault
        );
    }

    #[test]
    fn not_applicable_returns_default() {
        let mut i = base_pship_year();
        i.analysis_type = AnalysisType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_706_subsections() {
        let out = check(&base_pship_year());
        assert!(out.citations.iter().any(|c| c.contains("§ 706(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(b)(1)(B)(i)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(b)(1)(B)(ii)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(b)(1)(B)(iii)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(c)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(c)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 706(d)")));
    }

    #[test]
    fn citations_pin_treas_reg_1_706_1_and_4() {
        let out = check(&base_pship_year());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.706-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.706-4")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.706-4(f)")));
    }

    #[test]
    fn constant_pin_majority_50_pct() {
        assert_eq!(MAJORITY_INTEREST_TEST_THRESHOLD_PERCENT, 50);
    }

    #[test]
    fn constant_pin_principal_partner_5_pct() {
        assert_eq!(PRINCIPAL_PARTNER_TEST_THRESHOLD_PERCENT, 5);
    }

    #[test]
    fn constant_pin_de_minimis_half_month_x_10() {
        assert_eq!(DE_MINIMIS_LEAST_AGGREGATE_DEFERRAL_THRESHOLD_X_10, 5);
    }

    #[test]
    fn hierarchy_majority_takes_precedence_over_principal_partner() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = true;
        i.principal_partner_test_satisfied = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearMajorityInterestTestApplies
        );
    }

    #[test]
    fn hierarchy_principal_partner_takes_precedence_over_least_aggregate() {
        let mut i = base_pship_year();
        i.majority_interest_test_satisfied = false;
        i.principal_partner_test_satisfied = true;
        i.least_aggregate_deferral_change_months_x_10 = 20;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::PartnershipYearPrincipalPartnerTestApplies
        );
    }
}
