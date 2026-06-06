//! IRC § 6707 — Failure to furnish information regarding
//! reportable transactions. Material-advisor penalty
//! companion to section_6111 (Form 8918 material-advisor
//! disclosure obligation), section_6112 (material-advisor
//! list maintenance), section_6707a (TAXPAYER-side
//! disclosure penalty), section_6011 (Form 8886 taxpayer
//! disclosure), section_6662a (reportable-transaction
//! understatement accuracy penalty).
//!
//! Enacted by **American Jobs Creation Act of 2004 § 815**
//! (Pub. L. 108-357, **enacted October 22, 2004**) as part
//! of broader anti-shelter penalty regime created in
//! reaction to KPMG / E&Y / BDO Seidman shelter promotion
//! scandals of the early 2000s. § 6707 imposes the
//! MATERIAL ADVISOR-side civil penalty for failure to
//! comply with § 6111(a) Form 8918 disclosure requirement
//! within the prescribed time (last day of month after
//! calendar quarter in which advisor first becomes
//! material advisor).
//!
//! Trader-critical because aggressive tax shelter
//! promoters routinely advise on transactions that rise to
//! "listed transaction" status:
//! - **Basket option contracts** (Notice 2015-73) — synthetic
//!   tax-deferred trading vehicles used by hedge funds
//!   that received Treasury listed-transaction designation.
//! - **Conservation easement syndications** (Notice 2017-10)
//!   — promoter-driven inflated charitable deduction
//!   transactions.
//! - **Micro-captive insurance arrangements** (Notice
//!   2016-66 + listed status under 2024 final regulations).
//! - **Section 643 distribution-tier-out trusts**.
//! - **Structured trust advantaged repackaged securities
//!   (STARS)** and similar foreign-tax-credit shelters.
//!
//! Material advisor threshold under § 6111(b)(1) +
//! Treas. Reg. § 301.6111-3(b)(3):
//! - **$50,000** gross income on natural-person taxpayer
//!   transactions; OR
//! - **$250,000** gross income on entity/business taxpayer
//!   transactions.
//!
//! **§ 6707(a) Filing requirement cross-reference** — if
//! material advisor required to file return under § 6111
//! with respect to any reportable transaction FAILS to
//! file return on or before the date prescribed therefor,
//! OR FILES return that includes FALSE OR INCOMPLETE
//! information, such material advisor shall pay penalty
//! with respect to such return.
//!
//! **§ 6707(b)(1) Other reportable transactions — flat
//! $50,000 base penalty** — penalty for failure to comply
//! with § 6111 with respect to any reportable transaction
//! OTHER THAN a listed transaction is **$50,000**.
//!
//! **§ 6707(b)(2) Listed transactions — greater of
//! $200,000 OR percentage of gross income**:
//! 1. Penalty is GREATER of:
//!    - **$200,000**; OR
//!    - **50% of gross income** derived by material
//!      advisor from aid/assistance/advice provided.
//! 2. **INTENTIONAL failure or act**: 50% rate substituted
//!    with **75% of gross income**. Higher rate applies
//!    only to intentional failures.
//!
//! **§ 6707(c) Rescission authority**:
//! 1. § 6707(c)(1) — Commissioner may RESCIND penalty for
//!    OTHER reportable transactions (not listed) if
//!    rescission would promote tax compliance and
//!    effective tax administration.
//! 2. § 6707(c)(2) — **LISTED TRANSACTIONS are NOT
//!    eligible for rescission**. No exception, no waiver,
//!    no abatement absent statutory amendment.
//! 3. § 6707(c)(3) — **No judicial review** of denial of
//!    rescission request.
//!
//! **§ 6707(d) Coordination with § 7426** — penalty
//! interacts with related-party assessment provisions.
//!
//! **Reasonable cause defense — TWO TIERS**:
//! 1. **Listed transactions**: NO REASONABLE CAUSE
//!    DEFENSE AVAILABLE per § 6664(d) — strict liability.
//! 2. **Other reportable transactions**: § 6664(d)
//!    reasonable cause + good faith defense available;
//!    must establish substantial authority + good faith
//!    belief + reasonable cause.
//!
//! Citations: 26 USC § 6707(a)-(d); American Jobs Creation
//! Act of 2004 § 815 (Pub. L. 108-357, October 22, 2004);
//! 26 CFR § 301.6707-1; 26 USC § 6111; 26 USC § 6112;
//! 26 USC § 6707A; 26 USC § 6664(d); IRM 20.1.13 (Material
//! Advisor and Reportable Transactions Penalties); Form
//! 8918 (Material Advisor Disclosure Statement); Notice
//! 2015-73 (basket option contracts); Notice 2017-10
//! (conservation easement syndications); Notice 2016-66
//! (micro-captive insurance).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionCategory {
    /// § 6707(b)(2) — Listed transaction (Notice 2015-73,
    /// Notice 2017-10, Notice 2016-66, etc.).
    ListedTransaction,
    /// § 6707(b)(1) — Other reportable transaction
    /// (confidential, contractual protection, loss,
    /// transaction of interest, etc.).
    OtherReportableTransaction,
    /// Not a reportable transaction.
    NotReportable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6707Input {
    pub transaction_category: TransactionCategory,
    /// Whether material advisor filed Form 8918 by § 6111
    /// deadline (last day of month following quarter).
    pub form_8918_filed_timely: bool,
    /// Whether Form 8918 contained FALSE OR INCOMPLETE
    /// information.
    pub false_or_incomplete_information: bool,
    /// Gross income derived by material advisor from
    /// aid/assistance/advice with respect to the
    /// transaction in cents.
    pub gross_income_from_advice_cents: u64,
    /// Whether failure was INTENTIONAL (triggers 75% rate
    /// substitution on listed transactions).
    pub intentional_failure: bool,
    /// Whether reasonable cause + good faith defense
    /// engaged under § 6664(d) (only available for
    /// non-listed reportable transactions).
    pub reasonable_cause_engaged: bool,
    /// Whether Commissioner has issued rescission decision
    /// under § 6707(c)(1) (only available for non-listed
    /// transactions).
    pub commissioner_rescission_granted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6707Result {
    pub transaction_category: TransactionCategory,
    pub penalty_engaged: bool,
    pub penalty_cents: u64,
    pub intentional_75_percent_rate_engaged: bool,
    pub rescission_available: bool,
    pub reasonable_cause_defense_available: bool,
    pub judicial_review_of_rescission_denial_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6707Input) -> Section6707Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let violation_triggered = match input.transaction_category {
        TransactionCategory::NotReportable => false,
        _ => !input.form_8918_filed_timely || input.false_or_incomplete_information,
    };

    let (
        penalty_cents,
        intentional_75_percent_rate_engaged,
        reasonable_cause_defense_available,
        rescission_available,
    ) = if !violation_triggered {
        (0_u64, false, false, false)
    } else {
        match input.transaction_category {
            TransactionCategory::ListedTransaction => {
                let pct_bps: u32 = if input.intentional_failure {
                    7500
                } else {
                    5000
                };
                let pct_amount = input
                    .gross_income_from_advice_cents
                    .saturating_mul(pct_bps as u64)
                    / 10_000;
                let floor_cents: u64 = 20_000_000;
                let penalty = pct_amount.max(floor_cents);
                (penalty, input.intentional_failure, false, false)
            }
            TransactionCategory::OtherReportableTransaction => {
                let base = 5_000_000_u64;
                let final_penalty =
                    if input.commissioner_rescission_granted || input.reasonable_cause_engaged {
                        0
                    } else {
                        base
                    };
                (final_penalty, false, true, true)
            }
            TransactionCategory::NotReportable => (0, false, false, false),
        }
    };

    let penalty_engaged = penalty_cents > 0;
    let judicial_review_of_rescission_denial_available = false;

    if violation_triggered {
        match input.transaction_category {
            TransactionCategory::ListedTransaction => {
                failure_reasons.push(
                    "26 USC § 6707(b)(2) — listed-transaction material-advisor penalty: GREATER of $200,000 OR 50% (75% if INTENTIONAL) of gross income derived from aid/assistance/advice".to_string(),
                );
                failure_reasons.push(
                    "26 USC § 6707(c)(2) — listed transactions are NOT ELIGIBLE FOR RESCISSION; § 6664(d) reasonable cause defense NOT AVAILABLE — strict liability for listed-transaction material advisors".to_string(),
                );
                if input.intentional_failure {
                    failure_reasons.push(
                        "26 USC § 6707(b)(2) flush language — 50% rate SUBSTITUTED with 75% when failure or act is INTENTIONAL; higher rate applies only to intentional failures".to_string(),
                    );
                }
            }
            TransactionCategory::OtherReportableTransaction => {
                if !input.reasonable_cause_engaged && !input.commissioner_rescission_granted {
                    failure_reasons.push(
                        "26 USC § 6707(b)(1) — non-listed reportable-transaction material-advisor penalty: FLAT $50,000 for failure to comply with § 6111 Form 8918 disclosure within prescribed time".to_string(),
                    );
                } else if input.commissioner_rescission_granted {
                    failure_reasons.push(
                        "26 USC § 6707(c)(1) — Commissioner rescission granted on non-listed reportable transaction; penalty zeroed".to_string(),
                    );
                } else if input.reasonable_cause_engaged {
                    failure_reasons.push(
                        "26 USC § 6664(d) — reasonable cause + good faith defense engaged on non-listed reportable transaction; penalty zeroed (requires substantial authority + good faith belief + reasonable cause)".to_string(),
                    );
                }
            }
            TransactionCategory::NotReportable => {}
        }
    }

    let notes: Vec<String> = vec![
        "26 USC § 6707(a) — if material advisor required to file return under § 6111 with respect to ANY reportable transaction FAILS to file return on or before the date prescribed OR FILES return that includes FALSE OR INCOMPLETE INFORMATION, such material advisor shall pay penalty".to_string(),
        "26 USC § 6707(b)(1) — for OTHER REPORTABLE TRANSACTIONS (non-listed): FLAT $50,000 base penalty".to_string(),
        "26 USC § 6707(b)(2) — for LISTED TRANSACTIONS: GREATER of $200,000 OR 50% of gross income derived from aid/assistance/advice; 50% rate SUBSTITUTED with 75% when failure or act is INTENTIONAL".to_string(),
        "26 USC § 6707(c)(1) — Commissioner may RESCIND penalty for OTHER reportable transactions (not listed) if rescission promotes tax compliance and effective tax administration".to_string(),
        "26 USC § 6707(c)(2) — LISTED TRANSACTIONS ARE NOT ELIGIBLE FOR RESCISSION; no exception, no waiver, no abatement absent statutory amendment".to_string(),
        "26 USC § 6707(c)(3) — NO JUDICIAL REVIEW of denial of rescission request; Commissioner's denial is final".to_string(),
        "26 USC § 6664(d) reasonable cause + good faith defense — AVAILABLE for OTHER reportable transactions; NOT AVAILABLE for LISTED TRANSACTIONS (strict liability)".to_string(),
        "Enacted by American Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22, 2004) — broader anti-shelter penalty regime created in reaction to KPMG / E&Y / BDO Seidman shelter promotion scandals of early 2000s".to_string(),
        "Material advisor threshold under § 6111(b)(1) + Treas. Reg. § 301.6111-3(b)(3): $50,000 gross income (natural-person taxpayer transactions) OR $250,000 gross income (entity/business taxpayer transactions)".to_string(),
        "Form 8918 (Material Advisor Disclosure Statement) — filed with IRS Office of Tax Shelter Analysis (OTSA) by last day of month following calendar quarter in which advisor first becomes material advisor".to_string(),
        "Listed transaction examples include: Notice 2015-73 (basket option contracts); Notice 2017-10 (conservation easement syndications); Notice 2016-66 (micro-captive insurance); § 643 distribution-tier-out trusts; STARS foreign-tax-credit shelters".to_string(),
        "26 CFR § 301.6707-1 — implementing regulations on Material Advisor Penalty for Failure to Furnish Information Regarding Reportable Transactions; finalized in 2014 (T.D. 9683)".to_string(),
        "IRM 20.1.13 (Material Advisor and Reportable Transactions Penalties) — internal IRS guidance on § 6707 + § 6707A + § 6708 + § 6111 + § 6112 enforcement coordination".to_string(),
    ];

    Section6707Result {
        transaction_category: input.transaction_category,
        penalty_engaged,
        penalty_cents,
        intentional_75_percent_rate_engaged,
        rescission_available,
        reasonable_cause_defense_available,
        judicial_review_of_rescission_denial_available,
        failure_reasons,
        citation: "26 USC § 6707(a)-(d); American Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22, 2004); 26 CFR § 301.6707-1; 26 USC § 6111; 26 USC § 6112; 26 USC § 6707A; 26 USC § 6664(d); IRM 20.1.13; Form 8918; Notice 2015-73; Notice 2017-10; Notice 2016-66",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn other_reportable_violation() -> Section6707Input {
        Section6707Input {
            transaction_category: TransactionCategory::OtherReportableTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 100_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        }
    }

    #[test]
    fn other_reportable_flat_50k_penalty() {
        let r = check(&other_reportable_violation());
        assert!(r.penalty_engaged);
        assert_eq!(r.penalty_cents, 5_000_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6707(b)(1)") && f.contains("FLAT $50,000")));
    }

    #[test]
    fn other_reportable_reasonable_cause_zeros_penalty() {
        let mut i = other_reportable_violation();
        i.reasonable_cause_engaged = true;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 0);
        assert!(r.reasonable_cause_defense_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6664(d)") && f.contains("reasonable cause")));
    }

    #[test]
    fn other_reportable_commissioner_rescission_zeros_penalty() {
        let mut i = other_reportable_violation();
        i.commissioner_rescission_granted = true;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 0);
        assert!(r.rescission_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6707(c)(1)") && f.contains("Commissioner rescission granted")));
    }

    #[test]
    fn other_reportable_false_information_triggers_penalty() {
        let mut i = other_reportable_violation();
        i.form_8918_filed_timely = true;
        i.false_or_incomplete_information = true;
        let r = check(&i);
        assert!(r.penalty_engaged);
    }

    #[test]
    fn listed_transaction_unintentional_greater_of_200k_or_50_percent() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 500_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let r = check(&i);
        assert_eq!(r.penalty_cents, 250_000_000);
        assert!(!r.intentional_75_percent_rate_engaged);
    }

    #[test]
    fn listed_transaction_low_gross_income_floor_200k() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 10_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let r = check(&i);
        assert_eq!(r.penalty_cents, 20_000_000);
    }

    #[test]
    fn listed_transaction_intentional_75_percent_rate() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 1_000_000_000,
            intentional_failure: true,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let r = check(&i);
        assert_eq!(r.penalty_cents, 750_000_000);
        assert!(r.intentional_75_percent_rate_engaged);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6707(b)(2) flush language")
                && f.contains("75% when failure or act is INTENTIONAL")));
    }

    #[test]
    fn listed_transaction_no_rescission_available() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 100_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: true,
        };
        let r = check(&i);
        assert!(!r.rescission_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6707(c)(2)") && f.contains("NOT ELIGIBLE FOR RESCISSION")));
    }

    #[test]
    fn listed_transaction_no_reasonable_cause_defense() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 100_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: true,
            commissioner_rescission_granted: false,
        };
        let r = check(&i);
        assert!(!r.reasonable_cause_defense_available);
        assert!(r.penalty_cents >= 20_000_000);
    }

    #[test]
    fn not_reportable_no_penalty() {
        let mut i = other_reportable_violation();
        i.transaction_category = TransactionCategory::NotReportable;
        let r = check(&i);
        assert!(!r.penalty_engaged);
        assert_eq!(r.penalty_cents, 0);
    }

    #[test]
    fn no_judicial_review_of_rescission_denial() {
        let r = check(&other_reportable_violation());
        assert!(!r.judicial_review_of_rescission_denial_available);
    }

    #[test]
    fn intentional_listed_75_percent_uniquely_engages_invariant() {
        let make = |cat, intentional| Section6707Input {
            transaction_category: cat,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 1_000_000_000,
            intentional_failure: intentional,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let listed_intentional = check(&make(TransactionCategory::ListedTransaction, true));
        let listed_unintentional = check(&make(TransactionCategory::ListedTransaction, false));
        let other_intentional = check(&make(TransactionCategory::OtherReportableTransaction, true));
        assert!(listed_intentional.intentional_75_percent_rate_engaged);
        assert!(!listed_unintentional.intentional_75_percent_rate_engaged);
        assert!(!other_intentional.intentional_75_percent_rate_engaged);
        assert!(listed_intentional.penalty_cents > listed_unintentional.penalty_cents);
    }

    #[test]
    fn listed_uniquely_disables_reasonable_cause_defense_invariant() {
        let listed = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 100_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: true,
            commissioner_rescission_granted: false,
        };
        let r_listed = check(&listed);
        let r_other = check(&other_reportable_violation());
        assert!(!r_listed.reasonable_cause_defense_available);
        assert!(r_other.reasonable_cause_defense_available);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&other_reportable_violation());
        assert!(r.citation.contains("§ 6707(a)-(d)"));
        assert!(r
            .citation
            .contains("American Jobs Creation Act of 2004 § 815"));
        assert!(r.citation.contains("Pub. L. 108-357"));
        assert!(r.citation.contains("October 22, 2004"));
        assert!(r.citation.contains("26 CFR § 301.6707-1"));
        assert!(r.citation.contains("§ 6111"));
        assert!(r.citation.contains("§ 6112"));
        assert!(r.citation.contains("§ 6707A"));
        assert!(r.citation.contains("§ 6664(d)"));
        assert!(r.citation.contains("IRM 20.1.13"));
        assert!(r.citation.contains("Form 8918"));
        assert!(r.citation.contains("Notice 2015-73"));
        assert!(r.citation.contains("Notice 2017-10"));
        assert!(r.citation.contains("Notice 2016-66"));
    }

    #[test]
    fn note_pins_subsection_a_filing_requirement() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6707(a)")
            && n.contains("FAILS to file return")
            && n.contains("FALSE OR INCOMPLETE INFORMATION")));
    }

    #[test]
    fn note_pins_subsection_b1_flat_50k() {
        let r = check(&other_reportable_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6707(b)(1)") && n.contains("FLAT $50,000")));
    }

    #[test]
    fn note_pins_subsection_b2_greater_of_200k_or_50_percent_75_percent() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6707(b)(2)")
            && n.contains("GREATER of $200,000 OR 50%")
            && n.contains("SUBSTITUTED with 75%")
            && n.contains("INTENTIONAL")));
    }

    #[test]
    fn note_pins_subsection_c1_rescission_for_other_only() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6707(c)(1)")
            && n.contains("RESCIND")
            && n.contains("tax compliance")
            && n.contains("effective tax administration")));
    }

    #[test]
    fn note_pins_subsection_c2_listed_no_rescission() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6707(c)(2)")
            && n.contains("LISTED TRANSACTIONS ARE NOT ELIGIBLE FOR RESCISSION")));
    }

    #[test]
    fn note_pins_subsection_c3_no_judicial_review() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6707(c)(3)")
            && n.contains("NO JUDICIAL REVIEW")
            && n.contains("Commissioner's denial is final")));
    }

    #[test]
    fn note_pins_6664d_two_tier_reasonable_cause() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("§ 6664(d)")
            && n.contains("AVAILABLE for OTHER reportable transactions")
            && n.contains("NOT AVAILABLE for LISTED TRANSACTIONS")
            && n.contains("strict liability")));
    }

    #[test]
    fn note_pins_2004_ajca_origin() {
        let r = check(&other_reportable_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("American Jobs Creation Act of 2004 § 815")
                && n.contains("Pub. L. 108-357")
                && n.contains("October 22, 2004")
                && n.contains("KPMG")));
    }

    #[test]
    fn note_pins_material_advisor_threshold_50k_250k() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n
            .contains("Material advisor threshold under § 6111(b)(1)")
            && n.contains("$50,000")
            && n.contains("$250,000")));
    }

    #[test]
    fn note_pins_form_8918_quarterly_filing() {
        let r = check(&other_reportable_violation());
        assert!(r.notes.iter().any(|n| n.contains("Form 8918")
            && n.contains("OTSA")
            && n.contains("calendar quarter")));
    }

    #[test]
    fn note_pins_listed_transaction_examples() {
        let r = check(&other_reportable_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Listed transaction examples")
                && n.contains("Notice 2015-73")
                && n.contains("Notice 2017-10")
                && n.contains("Notice 2016-66")));
    }

    #[test]
    fn note_pins_301_6707_1_implementing_regulations() {
        let r = check(&other_reportable_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("26 CFR § 301.6707-1") && n.contains("T.D. 9683")));
    }

    #[test]
    fn note_pins_irm_20_1_13() {
        let r = check(&other_reportable_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("IRM 20.1.13") && n.contains("§ 6707A") && n.contains("§ 6708")));
    }

    #[test]
    fn transaction_category_truth_table_three_cells() {
        for cat in [
            TransactionCategory::ListedTransaction,
            TransactionCategory::OtherReportableTransaction,
            TransactionCategory::NotReportable,
        ] {
            let mut i = other_reportable_violation();
            i.transaction_category = cat;
            let r = check(&i);
            assert_eq!(r.transaction_category, cat);
        }
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let i = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: u64::MAX,
            intentional_failure: true,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let r = check(&i);
        let _ = r.penalty_cents;
        assert!(r.penalty_engaged);
    }

    #[test]
    fn listed_penalty_uniquely_uses_percentage_invariant() {
        let listed = Section6707Input {
            transaction_category: TransactionCategory::ListedTransaction,
            form_8918_filed_timely: false,
            false_or_incomplete_information: false,
            gross_income_from_advice_cents: 100_000_000,
            intentional_failure: false,
            reasonable_cause_engaged: false,
            commissioner_rescission_granted: false,
        };
        let other = other_reportable_violation();
        let r_listed = check(&listed);
        let r_other = check(&other);
        assert_eq!(r_listed.penalty_cents, 50_000_000);
        assert_eq!(r_other.penalty_cents, 5_000_000);
        assert!(r_listed.penalty_cents > r_other.penalty_cents);
    }
}
