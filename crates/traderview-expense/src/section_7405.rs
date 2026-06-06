//! IRC § 7405 — Action for recovery of erroneous refunds.
//! Trader-relevant when IRS issues refund and later
//! determines it was erroneous and seeks recovery via civil
//! action in district court. Companion to § 7422 (refund suit
//! filed by taxpayer) — § 7405 is the IRS-side reverse
//! mechanism. Trader-traders with Section 475(f) MTM election
//! receive substantial refunds; if IRS subsequently
//! determines NOL carryback computation was erroneous, § 7405
//! provides recovery pathway.
//!
//! Procedural-companion to § 7421 (Anti-Injunction Act), §
//! 7422 (taxpayer-initiated refund suit — reverse direction),
//! § 7426 (third-party wrongful levy), § 7429 (jeopardy
//! review), § 7430 (litigation costs), § 7433 (civil damages
//! for unauthorized collection), § 6514 (refunds otherwise
//! erroneous), and § 6532 (refund-suit limitations).
//!
//! **§ 7405(a) recovery of § 6514-erroneous refunds** — any
//! portion of a tax imposed by this title, refund of which
//! is erroneously made within the meaning of § 6514, may be
//! recovered by civil action brought in the name of the
//! United States.
//!
//! **§ 7405(b) recovery of non-§ 6514 erroneous refunds** —
//! any portion of tax erroneously refunded (if such refund
//! would NOT be considered erroneous under § 6514) may also
//! be recovered by civil action in the name of the United
//! States.
//!
//! **§ 7405(d) statute of limitations**:
//! - **2 years** from the making of the refund (standard
//!   SOL), OR
//! - **5 years** if the refund was induced by FRAUD or
//!   MISREPRESENTATION of a material fact.
//!
//! **IRS burden of proof** (per IRM 5.17.4 + case law) —
//! government must establish:
//! 1. The refund was erroneous.
//! 2. The amount of the refund.
//! 3. The taxpayer received or benefited from the erroneous
//!    refund.
//!
//! **Jurisdiction**: district court (concurrent with United
//! States Court of Federal Claims under 28 USC § 1346).
//!
//! Citations: 26 USC § 7405(a), (b), (c), (d); § 6514
//! (refunds otherwise erroneous); IRM 5.17.4 (Suits by the
//! United States); 28 USC § 1346(a)(1) (concurrent
//! jurisdiction).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RefundType {
    /// § 7405(a) — refund erroneous within meaning of §
    /// 6514.
    Section6514Erroneous,
    /// § 7405(b) — refund erroneously made but NOT § 6514
    /// erroneous.
    NonSection6514Erroneous,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7405Input {
    pub refund_type: RefundType,
    /// Days since IRS made the refund (for § 7405(d) SOL).
    pub days_since_refund: u32,
    /// Whether refund was induced by fraud or
    /// misrepresentation of material fact (engages 5-year
    /// SOL).
    pub fraud_or_misrepresentation: bool,
    /// Whether IRS has established refund was erroneous
    /// (burden of proof element 1).
    pub refund_proven_erroneous: bool,
    /// Whether IRS has established amount of refund (burden
    /// of proof element 2).
    pub amount_established: bool,
    /// Whether IRS has established taxpayer received or
    /// benefited from refund (burden of proof element 3).
    pub taxpayer_received_or_benefited: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7405Result {
    pub recovery_action_maintainable: bool,
    pub applicable_sol_days: u32,
    pub sol_satisfied: bool,
    pub fraud_extends_sol: bool,
    pub burden_of_proof_satisfied: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7405Input) -> Section7405Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let applicable_sol: u32 = if input.fraud_or_misrepresentation {
        1825
    } else {
        730
    };

    let sol_satisfied = input.days_since_refund <= applicable_sol;

    if !sol_satisfied {
        failure_reasons.push(format!(
            "26 USC § 7405(d) — recovery action filed {} days after refund; exceeds {}-day SOL ({})",
            input.days_since_refund,
            applicable_sol,
            if input.fraud_or_misrepresentation {
                "5-year fraud / misrepresentation SOL"
            } else {
                "standard 2-year SOL"
            }
        ));
    }

    let burden_complete = input.refund_proven_erroneous
        && input.amount_established
        && input.taxpayer_received_or_benefited;

    if !input.refund_proven_erroneous {
        failure_reasons.push(
            "IRM 5.17.4 + § 7405 case law — IRS burden of proof element 1: must establish refund was erroneous"
                .to_string(),
        );
    }
    if !input.amount_established {
        failure_reasons.push(
            "IRM 5.17.4 + § 7405 case law — IRS burden of proof element 2: must establish amount of refund"
                .to_string(),
        );
    }
    if !input.taxpayer_received_or_benefited {
        failure_reasons.push(
            "IRM 5.17.4 + § 7405 case law — IRS burden of proof element 3: must establish taxpayer received or benefited from erroneous refund"
                .to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 7405(a) — IRS may recover refund erroneous within meaning of § 6514 by civil action in name of United States"
            .to_string(),
        "26 USC § 7405(b) — IRS may recover refund erroneously made even if NOT § 6514 erroneous (reaches refunds outside § 6514 scope)"
            .to_string(),
        "26 USC § 7405(d) statute of limitations — 2 years (730 days) from making of refund standard; 5 years (1825 days) if refund induced by fraud or misrepresentation of material fact"
            .to_string(),
        "IRS burden of proof per IRM 5.17.4 — government must establish: (1) refund was erroneous; (2) amount of refund; (3) taxpayer received or benefited from refund"
            .to_string(),
        "Jurisdiction — district court (concurrent with United States Court of Federal Claims under 28 USC § 1346(a)(1)); § 7405 is reverse-direction companion to § 7422 (taxpayer-initiated refund suit)"
            .to_string(),
    ];

    Section7405Result {
        recovery_action_maintainable: failure_reasons.is_empty(),
        applicable_sol_days: applicable_sol,
        sol_satisfied,
        fraud_extends_sol: input.fraud_or_misrepresentation,
        burden_of_proof_satisfied: burden_complete,
        failure_reasons,
        citation: "26 USC § 7405(a), (b), (c), (d); § 6514; IRM 5.17.4; 28 USC § 1346(a)(1)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn maintainable_base() -> Section7405Input {
        Section7405Input {
            refund_type: RefundType::Section6514Erroneous,
            days_since_refund: 365,
            fraud_or_misrepresentation: false,
            refund_proven_erroneous: true,
            amount_established: true,
            taxpayer_received_or_benefited: true,
        }
    }

    fn fraud_base() -> Section7405Input {
        let mut i = maintainable_base();
        i.fraud_or_misrepresentation = true;
        i.days_since_refund = 1000;
        i
    }

    #[test]
    fn fully_compliant_2_year_action_maintainable() {
        let r = check(&maintainable_base());
        assert!(r.recovery_action_maintainable);
        assert!(r.sol_satisfied);
        assert_eq!(r.applicable_sol_days, 730);
        assert!(!r.fraud_extends_sol);
        assert!(r.burden_of_proof_satisfied);
    }

    #[test]
    fn at_730_day_2_year_boundary_compliant() {
        let mut i = maintainable_base();
        i.days_since_refund = 730;
        let r = check(&i);
        assert!(r.recovery_action_maintainable);
        assert!(r.sol_satisfied);
    }

    #[test]
    fn at_731_days_2_year_violates() {
        let mut i = maintainable_base();
        i.days_since_refund = 731;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert!(!r.sol_satisfied);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 7405(d)")
            && f.contains("731")
            && f.contains("standard 2-year")));
    }

    #[test]
    fn fraud_at_1000_days_compliant() {
        let r = check(&fraud_base());
        assert!(r.recovery_action_maintainable);
        assert_eq!(r.applicable_sol_days, 1825);
        assert!(r.fraud_extends_sol);
    }

    #[test]
    fn fraud_at_1825_day_boundary_compliant() {
        let mut i = fraud_base();
        i.days_since_refund = 1825;
        let r = check(&i);
        assert!(r.recovery_action_maintainable);
    }

    #[test]
    fn fraud_at_1826_days_violates() {
        let mut i = fraud_base();
        i.days_since_refund = 1826;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7405(d)") && f.contains("1826") && f.contains("5-year")));
    }

    #[test]
    fn non_section_6514_recovery_action_maintainable() {
        let mut i = maintainable_base();
        i.refund_type = RefundType::NonSection6514Erroneous;
        let r = check(&i);
        assert!(r.recovery_action_maintainable);
    }

    #[test]
    fn no_refund_proven_erroneous_violates() {
        let mut i = maintainable_base();
        i.refund_proven_erroneous = false;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert!(!r.burden_of_proof_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 1") && f.contains("erroneous")));
    }

    #[test]
    fn no_amount_established_violates() {
        let mut i = maintainable_base();
        i.amount_established = false;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 2") && f.contains("amount")));
    }

    #[test]
    fn no_taxpayer_received_violates() {
        let mut i = maintainable_base();
        i.taxpayer_received_or_benefited = false;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("element 3") && f.contains("received or benefited")));
    }

    #[test]
    fn all_burden_elements_failed_3_violations_stack() {
        let mut i = maintainable_base();
        i.refund_proven_erroneous = false;
        i.amount_established = false;
        i.taxpayer_received_or_benefited = false;
        let r = check(&i);
        assert!(!r.recovery_action_maintainable);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&maintainable_base());
        assert!(r.citation.contains("§ 7405(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("§ 6514"));
        assert!(r.citation.contains("IRM 5.17.4"));
        assert!(r.citation.contains("§ 1346(a)(1)"));
    }

    #[test]
    fn note_pins_section_7405a_b_recovery_paths() {
        let r = check(&maintainable_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7405(a)") && n.contains("§ 6514")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7405(b)") && n.contains("NOT § 6514")));
    }

    #[test]
    fn note_pins_2_5_year_sol_framework() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7405(d)")
            && n.contains("2 years (730 days)")
            && n.contains("5 years (1825 days)")
            && n.contains("fraud or misrepresentation")));
    }

    #[test]
    fn note_pins_irs_burden_of_proof_three_elements() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("burden of proof")
            && n.contains("IRM 5.17.4")
            && n.contains("(1) refund was erroneous")
            && n.contains("(2) amount")
            && n.contains("(3) taxpayer")));
    }

    #[test]
    fn note_pins_jurisdiction_and_companion_section_7422() {
        let r = check(&maintainable_base());
        assert!(r.notes.iter().any(|n| n.contains("district court")
            && n.contains("§ 1346(a)(1)")
            && n.contains("§ 7422")));
    }

    #[test]
    fn fraud_truth_table_extends_sol() {
        for (fraud, exp_sol) in [(false, 730_u32), (true, 1825_u32)] {
            let mut i = maintainable_base();
            i.fraud_or_misrepresentation = fraud;
            let r = check(&i);
            assert_eq!(r.applicable_sol_days, exp_sol);
            assert_eq!(r.fraud_extends_sol, fraud);
        }
    }

    #[test]
    fn fraud_uniquely_extends_sol_invariant() {
        let mut i_no_fraud = maintainable_base();
        i_no_fraud.fraud_or_misrepresentation = false;
        i_no_fraud.days_since_refund = 1000;
        let r_no_fraud = check(&i_no_fraud);
        assert!(!r_no_fraud.recovery_action_maintainable);

        let mut i_fraud = maintainable_base();
        i_fraud.fraud_or_misrepresentation = true;
        i_fraud.days_since_refund = 1000;
        let r_fraud = check(&i_fraud);
        assert!(r_fraud.recovery_action_maintainable);
    }

    #[test]
    fn refund_type_truth_table() {
        let mut i_a = maintainable_base();
        i_a.refund_type = RefundType::Section6514Erroneous;
        let r_a = check(&i_a);
        assert!(r_a.recovery_action_maintainable);

        let mut i_b = maintainable_base();
        i_b.refund_type = RefundType::NonSection6514Erroneous;
        let r_b = check(&i_b);
        assert!(r_b.recovery_action_maintainable);
    }

    #[test]
    fn burden_of_proof_truth_table() {
        for (e1, e2, e3, exp_burden) in [
            (true, true, true, true),
            (false, true, true, false),
            (true, false, true, false),
            (true, true, false, false),
            (false, false, false, false),
        ] {
            let mut i = maintainable_base();
            i.refund_proven_erroneous = e1;
            i.amount_established = e2;
            i.taxpayer_received_or_benefited = e3;
            let r = check(&i);
            assert_eq!(r.burden_of_proof_satisfied, exp_burden);
        }
    }

    #[test]
    fn at_730_day_2_year_boundary_with_fraud_engages_5_year_window() {
        let mut i = maintainable_base();
        i.days_since_refund = 730;
        i.fraud_or_misrepresentation = true;
        let r = check(&i);
        assert!(r.recovery_action_maintainable);
        assert_eq!(r.applicable_sol_days, 1825);
    }

    #[test]
    fn precise_1_day_boundary_distinguishes_2_year_vs_fraud() {
        let mut i_731_no_fraud = maintainable_base();
        i_731_no_fraud.days_since_refund = 731;
        i_731_no_fraud.fraud_or_misrepresentation = false;
        let r1 = check(&i_731_no_fraud);
        assert!(!r1.recovery_action_maintainable);

        let mut i_731_fraud = maintainable_base();
        i_731_fraud.days_since_refund = 731;
        i_731_fraud.fraud_or_misrepresentation = true;
        let r2 = check(&i_731_fraud);
        assert!(r2.recovery_action_maintainable);
    }

    #[test]
    fn citations_field_includes_irm() {
        let r = check(&maintainable_base());
        assert!(r.citation.contains("IRM 5.17.4"));
    }

    #[test]
    fn sol_satisfied_truth_table() {
        for (days, fraud, exp_satisfied) in [
            (0_u32, false, true),
            (730, false, true),
            (731, false, false),
            (1825, true, true),
            (1826, true, false),
            (0, true, true),
            (5000, true, false),
        ] {
            let mut i = maintainable_base();
            i.days_since_refund = days;
            i.fraud_or_misrepresentation = fraud;
            let r = check(&i);
            assert_eq!(r.sol_satisfied, exp_satisfied);
        }
    }

    #[test]
    fn standard_2_year_uniquely_lower_than_fraud_5_year_invariant() {
        let r_standard = check(&maintainable_base());
        let r_fraud = check(&fraud_base());
        assert!(r_standard.applicable_sol_days < r_fraud.applicable_sol_days);
        assert_eq!(r_fraud.applicable_sol_days, 1825);
        assert_eq!(r_standard.applicable_sol_days, 730);
    }
}
