//! IRC §451(b) — AFS conformity / all-events test acceleration (TCJA).
//!
//! Added by TCJA § 13221 (P.L. 115-97), effective tax years
//! beginning after 2017-12-31. Requires accrual-method taxpayers
//! with an **Applicable Financial Statement (AFS)** to include an
//! item of income in gross income no LATER than when the item is
//! recognized as revenue on the AFS.
//!
//! This is a **ONE-DIRECTIONAL acceleration rule** — it only
//! advances the timing of income recognition for tax purposes; it
//! never defers ([FEI — Where GAAP and Tax Meet: § 451(b)](https://www.financialexecutives.org/FEI-Daily/October-2019/Where-GAAP-and-Tax-Meet-Understanding-IRC-451b.aspx),
//! [Cornell LII 26 U.S.C. § 451](https://www.law.cornell.edu/uscode/text/26/451)).
//!
//! **§ 451(b) all-events test**: item of income is recognized at the
//! EARLIEST of when it is:
//!
//! - Due (cash or accrual right to receive)
//! - Paid
//! - Earned (classic all-events test)
//! - Taken into account as revenue in the taxpayer's AFS
//!
//! **§ 451(b)(3) AFS definition (hierarchy, highest first)**:
//!
//! 1. Financial statement filed with the SEC (10-K, 10-Q, etc.)
//! 2. Audited financial statement used for credit, financial
//!    reporting, or other substantial nontax purposes
//! 3. Other independently audited financial statements
//!
//! Taxpayers without an AFS are NOT subject to § 451(b); they apply
//! the classic all-events test under § 451(a).
//!
//! **§ 451(b) coincides with ASC 606** — FASB Accounting Standards
//! Codification Topic 606 (Revenue From Contracts With Customers,
//! effective 2018 public / 2019 private) often accelerates GAAP
//! revenue recognition, which in turn accelerates tax recognition
//! under § 451(b).
//!
//! **§ 451(b) cost offset election** (TD 9941, eff. 2020-12-21):
//! optional method allowing taxpayers to offset AFS income inclusions
//! for over-time revenue recognition with costs incurred to date.
//! Mitigates the harsh result of accelerated income inclusion without
//! corresponding COGS deduction.
//!
//! **§ 451(c) advance payment deferral**: codifies Rev. Proc.
//! 2004-34's 1-year deferral allowance for advance payments
//! (deferred revenue). Accrual taxpayer may defer the portion of an
//! advance payment not recognized on AFS in year of receipt until
//! the FOLLOWING tax year.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section451bInput {
    pub tax_year: i32,
    /// True if taxpayer uses an overall accrual method of accounting.
    pub uses_accrual_method: bool,
    /// True if taxpayer has an Applicable Financial Statement under
    /// § 451(b)(3) — SEC-filed, audited, or independent-CPA-audited.
    pub has_applicable_financial_statement: bool,
    pub afs_revenue_recognized_for_item_dollars: i64,
    pub classic_all_events_test_amount_dollars: i64,
    /// Costs incurred to date that may be applied as an offset under
    /// the § 451(b) cost offset election (TD 9941). Only available
    /// for inventory sold under over-time AFS recognition.
    pub costs_incurred_to_date_for_cost_offset_dollars: i64,
    pub elects_section_451b_cost_offset: bool,
    /// True if taxpayer received an advance payment in the current
    /// year that has not yet been recognized as revenue on AFS.
    pub has_advance_payment: bool,
    pub advance_payment_received_current_year_dollars: i64,
    pub afs_advance_payment_recognized_current_year_dollars: i64,
    pub elects_section_451c_one_year_deferral: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section451bResult {
    pub section_451b_applies: bool,
    pub accelerated_inclusion_amount_dollars: i64,
    pub cost_offset_applied_dollars: i64,
    pub net_current_year_inclusion_dollars: i64,
    pub section_451c_advance_payment_deferred_to_next_year_dollars: i64,
    pub section_451c_advance_payment_current_year_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section451bInput) -> Section451bResult {
    // § 451(b) applies only when both conditions met AND tax year is
    // post-TCJA (2018 forward).
    let post_tcja = input.tax_year >= 2018;
    let applies =
        post_tcja && input.uses_accrual_method && input.has_applicable_financial_statement;

    // Acceleration: take the GREATER of AFS revenue and classic
    // all-events. § 451(b) one-directional rule — never decreases.
    let accelerated = if applies {
        input
            .afs_revenue_recognized_for_item_dollars
            .max(input.classic_all_events_test_amount_dollars)
    } else {
        input.classic_all_events_test_amount_dollars
    };

    // § 451(b) cost offset election (TD 9941). Caps offset at AFS
    // recognition; offset may not exceed accelerated inclusion.
    let cost_offset = if applies && input.elects_section_451b_cost_offset {
        input
            .costs_incurred_to_date_for_cost_offset_dollars
            .min(accelerated)
            .max(0)
    } else {
        0
    };
    let net_current = (accelerated - cost_offset).max(0);

    // § 451(c) advance payment deferral.
    let advance_current = if applies && input.has_advance_payment {
        if input.elects_section_451c_one_year_deferral {
            // Current year: amount recognized on AFS in current year.
            input.afs_advance_payment_recognized_current_year_dollars
        } else {
            // Without election, advance payment recognized in full
            // current year.
            input.advance_payment_received_current_year_dollars
        }
    } else {
        0
    };
    let advance_deferred =
        if applies && input.has_advance_payment && input.elects_section_451c_one_year_deferral {
            (input.advance_payment_received_current_year_dollars - advance_current).max(0)
        } else {
            0
        };

    let note = if !post_tcja {
        format!(
            "Tax year {} is pre-TCJA; § 451(b) not yet enacted. Classic all-events test applies: ${} inclusion.",
            input.tax_year, input.classic_all_events_test_amount_dollars,
        )
    } else if !input.uses_accrual_method {
        format!(
            "Cash-method taxpayer; § 451(b) does not apply. Income recognized when received: ${}.",
            input.afs_revenue_recognized_for_item_dollars,
        )
    } else if !input.has_applicable_financial_statement {
        format!(
            "Accrual taxpayer WITHOUT AFS; § 451(b) does not apply. Classic § 451(a) all-events test: ${}.",
            input.classic_all_events_test_amount_dollars,
        )
    } else {
        let mut parts = vec![format!(
            "§ 451(b) APPLIES: accrual + AFS taxpayer. Accelerated inclusion = max(AFS ${} , all-events ${}) = ${}.",
            input.afs_revenue_recognized_for_item_dollars,
            input.classic_all_events_test_amount_dollars,
            accelerated,
        )];
        if cost_offset > 0 {
            parts.push(format!(
                "§ 451(b) cost offset election (TD 9941): ${} costs incurred to date, ${} offset applied, ${} net current-year inclusion.",
                input.costs_incurred_to_date_for_cost_offset_dollars,
                cost_offset,
                net_current,
            ));
        }
        if input.has_advance_payment {
            if input.elects_section_451c_one_year_deferral {
                parts.push(format!(
                    "§ 451(c) 1-year deferral elected: ${} advance payment, ${} AFS-recognized current year, ${} deferred to next year.",
                    input.advance_payment_received_current_year_dollars,
                    advance_current,
                    advance_deferred,
                ));
            } else {
                parts.push(format!(
                    "Advance payment ${} recognized in full current year (no § 451(c) election).",
                    input.advance_payment_received_current_year_dollars,
                ));
            }
        }
        parts.join(" ")
    };

    Section451bResult {
        section_451b_applies: applies,
        accelerated_inclusion_amount_dollars: accelerated,
        cost_offset_applied_dollars: cost_offset,
        net_current_year_inclusion_dollars: net_current,
        section_451c_advance_payment_deferred_to_next_year_dollars: advance_deferred,
        section_451c_advance_payment_current_year_dollars: advance_current,
        citation:
            "IRC §451(b) AFS conformity rule (TCJA P.L. 115-97 §13221, eff. tax years beginning after 2017-12-31); §451(b)(3) AFS hierarchy (SEC-filed > audited > certified); §451(b) cost offset election under Treas. Reg. §1.451-3 (TD 9941, eff. 2020-12-21); §451(c) 1-year advance payment deferral (codifying Rev. Proc. 2004-34); FASB ASC 606 revenue recognition standard interaction"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section451bInput {
        Section451bInput {
            tax_year: 2025,
            uses_accrual_method: true,
            has_applicable_financial_statement: true,
            afs_revenue_recognized_for_item_dollars: 1_000_000,
            classic_all_events_test_amount_dollars: 800_000,
            costs_incurred_to_date_for_cost_offset_dollars: 0,
            elects_section_451b_cost_offset: false,
            has_advance_payment: false,
            advance_payment_received_current_year_dollars: 0,
            afs_advance_payment_recognized_current_year_dollars: 0,
            elects_section_451c_one_year_deferral: false,
        }
    }

    // § 451(b) applies → max of AFS / all-events.

    #[test]
    fn afs_higher_than_all_events_uses_afs() {
        // AFS $1M > all-events $800k → accelerated to $1M.
        let r = compute(&base());
        assert!(r.section_451b_applies);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 1_000_000);
    }

    #[test]
    fn all_events_higher_than_afs_uses_all_events() {
        let mut i = base();
        i.classic_all_events_test_amount_dollars = 1_500_000;
        let r = compute(&i);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 1_500_000);
    }

    #[test]
    fn one_directional_rule_never_decreases() {
        // AFS $500k < all-events $800k → keep $800k (acceleration only).
        let mut i = base();
        i.afs_revenue_recognized_for_item_dollars = 500_000;
        let r = compute(&i);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 800_000);
    }

    // § 451(b) does NOT apply.

    #[test]
    fn pre_tcja_year_no_afs_acceleration() {
        let mut i = base();
        i.tax_year = 2017;
        let r = compute(&i);
        assert!(!r.section_451b_applies);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 800_000);
        assert!(r.note.contains("pre-TCJA"));
    }

    #[test]
    fn cash_method_taxpayer_no_451b() {
        let mut i = base();
        i.uses_accrual_method = false;
        let r = compute(&i);
        assert!(!r.section_451b_applies);
    }

    #[test]
    fn no_afs_no_451b_applies() {
        let mut i = base();
        i.has_applicable_financial_statement = false;
        let r = compute(&i);
        assert!(!r.section_451b_applies);
        assert!(r.note.contains("WITHOUT AFS"));
    }

    // § 451(b) cost offset election.

    #[test]
    fn cost_offset_election_reduces_current_year_inclusion() {
        // $1M accelerated, $300k costs incurred → $700k net.
        let mut i = base();
        i.elects_section_451b_cost_offset = true;
        i.costs_incurred_to_date_for_cost_offset_dollars = 300_000;
        let r = compute(&i);
        assert_eq!(r.cost_offset_applied_dollars, 300_000);
        assert_eq!(r.net_current_year_inclusion_dollars, 700_000);
    }

    #[test]
    fn cost_offset_capped_at_accelerated_inclusion() {
        // Costs > accelerated → cap at accelerated.
        let mut i = base();
        i.elects_section_451b_cost_offset = true;
        i.costs_incurred_to_date_for_cost_offset_dollars = 5_000_000;
        let r = compute(&i);
        assert_eq!(r.cost_offset_applied_dollars, 1_000_000);
        assert_eq!(r.net_current_year_inclusion_dollars, 0);
    }

    #[test]
    fn cost_offset_not_applied_without_election() {
        let mut i = base();
        i.costs_incurred_to_date_for_cost_offset_dollars = 300_000;
        i.elects_section_451b_cost_offset = false;
        let r = compute(&i);
        assert_eq!(r.cost_offset_applied_dollars, 0);
        assert_eq!(r.net_current_year_inclusion_dollars, 1_000_000);
    }

    // § 451(c) advance payment deferral.

    #[test]
    fn advance_payment_without_election_fully_current_year() {
        let mut i = base();
        i.has_advance_payment = true;
        i.advance_payment_received_current_year_dollars = 500_000;
        i.afs_advance_payment_recognized_current_year_dollars = 100_000;
        i.elects_section_451c_one_year_deferral = false;
        let r = compute(&i);
        assert_eq!(r.section_451c_advance_payment_current_year_dollars, 500_000);
        assert_eq!(
            r.section_451c_advance_payment_deferred_to_next_year_dollars,
            0
        );
    }

    #[test]
    fn advance_payment_with_election_defers_unrecognized_portion() {
        // $500k received, $100k recognized on AFS current → $400k deferred.
        let mut i = base();
        i.has_advance_payment = true;
        i.advance_payment_received_current_year_dollars = 500_000;
        i.afs_advance_payment_recognized_current_year_dollars = 100_000;
        i.elects_section_451c_one_year_deferral = true;
        let r = compute(&i);
        assert_eq!(r.section_451c_advance_payment_current_year_dollars, 100_000);
        assert_eq!(
            r.section_451c_advance_payment_deferred_to_next_year_dollars,
            400_000
        );
    }

    #[test]
    fn advance_payment_fully_afs_recognized_no_deferral() {
        // $500k received, $500k AFS recognized → $500k current, $0 deferred.
        let mut i = base();
        i.has_advance_payment = true;
        i.advance_payment_received_current_year_dollars = 500_000;
        i.afs_advance_payment_recognized_current_year_dollars = 500_000;
        i.elects_section_451c_one_year_deferral = true;
        let r = compute(&i);
        assert_eq!(r.section_451c_advance_payment_current_year_dollars, 500_000);
        assert_eq!(
            r.section_451c_advance_payment_deferred_to_next_year_dollars,
            0
        );
    }

    // Notes / citations.

    #[test]
    fn applies_note_describes_max_formula() {
        let r = compute(&base());
        assert!(r.note.contains("§ 451(b) APPLIES"));
        assert!(r.note.contains("max(AFS"));
    }

    #[test]
    fn cost_offset_election_note_describes_td9941() {
        let mut i = base();
        i.elects_section_451b_cost_offset = true;
        i.costs_incurred_to_date_for_cost_offset_dollars = 300_000;
        let r = compute(&i);
        assert!(r.note.contains("§ 451(b) cost offset election (TD 9941)"));
    }

    #[test]
    fn advance_payment_election_note_describes_451c() {
        let mut i = base();
        i.has_advance_payment = true;
        i.advance_payment_received_current_year_dollars = 500_000;
        i.afs_advance_payment_recognized_current_year_dollars = 100_000;
        i.elects_section_451c_one_year_deferral = true;
        let r = compute(&i);
        assert!(r.note.contains("§ 451(c) 1-year deferral"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§451(b)"));
        assert!(r.citation.contains("§451(b)(3)"));
        assert!(r.citation.contains("§451(c)"));
        assert!(r.citation.contains("TCJA"));
        assert!(r.citation.contains("§13221"));
        assert!(r.citation.contains("TD 9941"));
        assert!(r.citation.contains("ASC 606"));
        assert!(r.citation.contains("Rev. Proc. 2004-34"));
    }

    // Precision / large.

    #[test]
    fn very_large_revenue_precision() {
        let mut i = base();
        i.afs_revenue_recognized_for_item_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 1_000_000_000);
    }

    #[test]
    fn zero_revenue_no_op() {
        let mut i = base();
        i.afs_revenue_recognized_for_item_dollars = 0;
        i.classic_all_events_test_amount_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.accelerated_inclusion_amount_dollars, 0);
        assert_eq!(r.net_current_year_inclusion_dollars, 0);
    }

    // 2018 boundary.

    #[test]
    fn tax_year_2018_first_post_tcja_year() {
        let mut i = base();
        i.tax_year = 2018;
        let r = compute(&i);
        assert!(r.section_451b_applies);
    }

    #[test]
    fn tax_year_2017_pre_tcja_does_not_apply() {
        let mut i = base();
        i.tax_year = 2017;
        let r = compute(&i);
        assert!(!r.section_451b_applies);
    }
}
