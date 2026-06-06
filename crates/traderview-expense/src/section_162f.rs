//! IRC § 162(f) — Fines and penalties; general rule of
//! nondeductibility for amounts paid in relation to violation of
//! law. Trader-relevant when a trader-business pays regulatory
//! fines (FINRA, SEC, CFTC, state securities regulators, exchange
//! disciplinary), tax penalties, environmental fines, or other
//! government-imposed punitive amounts.
//!
//! Distinct from `section_6651` (FTF/FTP penalties — which compute
//! the dollar amounts of those penalties; deductibility of the
//! penalties themselves runs through § 162(f)) and `section_6662`
//! (accuracy-related penalty). This module addresses ONLY the
//! DEDUCTIBILITY question for fines, penalties, and other amounts
//! paid in relation to violation of law.
//!
//! § 162(f)(1) GENERAL RULE (post-TCJA Dec 22, 2017) — no
//! deduction shall be allowed for any amount paid or incurred
//! (whether by suit, agreement, or otherwise) to, or at the
//! direction of, a government or governmental entity in relation
//! to the violation of any law OR the investigation or inquiry by
//! such government or entity into the potential violation of any
//! law.
//!
//! § 162(f)(2) EXCEPTION — paragraph (1) shall not apply to any
//! amount that:
//!   (A) the taxpayer establishes (1) constitutes RESTITUTION
//!       (including REMEDIATION of property) for damage or harm
//!       caused by the violation OR potential violation of any
//!       law, OR (2) is paid to come into COMPLIANCE with any
//!       law that was violated or otherwise involved in the
//!       investigation or inquiry, AND
//!   (B) the court order or settlement agreement IDENTIFIES the
//!       amount as restitution, remediation, or compliance.
//!
//! § 162(f)(2)(A)(ii) — IDENTIFICATION + ESTABLISHMENT — BOTH
//! prongs required. Court order or settlement agreement must
//! explicitly identify the amount and purpose; taxpayer must prove
//! that the payment was actually made for the identified purpose.
//!
//! § 162(f)(3) RULES OF APPLICATION — does NOT apply to (A)
//! routine investigation costs, (B) court costs, (C) etc.
//! Routine business expenses incurred during an investigation
//! remain § 162(a) deductible.
//!
//! § 162(f)(5) QUI TAM PAYMENTS — special rule for amounts paid
//! to a private relator under the False Claims Act or similar
//! qui tam statute; the relator is NOT a "government" for
//! § 162(f)(1) purposes.
//!
//! § 6050X INFORMATION REPORTING — governmental and entities
//! receiving payments described in § 162(f) of $50,000 or more
//! must file Form 1098-F with the IRS and furnish a copy to the
//! payor. Drives audit risk for taxpayers who deduct fines.
//!
//! EFFECTIVE DATE: § 162(f) as amended by TCJA applies to amounts
//! paid or incurred on or after December 22, 2017. Pre-existing
//! binding orders or agreements are grandfathered even if
//! modified after that date.
//!
//! § 162(q) — separate restriction enacted by TCJA: no deduction
//! for any settlement or payment related to sexual harassment or
//! sexual abuse if subject to a nondisclosure agreement. Out of
//! scope of this module; flagged in citation.
//!
//! Citations: IRC § 162(f)(1) (general nondeductibility rule);
//! § 162(f)(2)(A)(i) (restitution exception); § 162(f)(2)(A)(ii)
//! (compliance exception); § 162(f)(2)(B) (identification
//! requirement); § 162(f)(3) (routine investigation costs);
//! § 162(f)(5) (qui tam payments); IRC § 6050X (Form 1098-F
//! information reporting threshold $50,000); § 162(q) (sexual
//! harassment NDA settlements — separate restriction); TCJA
//! 2017 § 13306 (TCJA amendment to § 162(f)); Treas. Reg.
//! § 1.162-21 (TD 9946, January 19, 2021 final regulations).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    /// Pure punitive fine or civil penalty in relation to
    /// violation of law.
    Fine,
    /// Restitution (including remediation of property) for damage
    /// or harm caused by the violation. § 162(f)(2)(A)(i).
    RestitutionOrRemediation,
    /// Amount paid to come into compliance with the law that was
    /// violated or otherwise involved. § 162(f)(2)(A)(ii).
    ComplianceWithLaw,
    /// Routine investigation cost or court cost. § 162(f)(3).
    RoutineInvestigationOrCourtCost,
    /// Tax payment (separate framework — § 164).
    TaxPayment,
    /// Qui tam payment to a private relator under FCA or similar.
    /// § 162(f)(5).
    QuiTamPayment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section162fInput {
    pub payment_type: PaymentType,
    /// Whether the payment is to a government, governmental
    /// entity, or nongovernmental entity at the direction of the
    /// government in relation to the violation of law or
    /// investigation/inquiry.
    pub payment_to_government_in_relation_to_violation: bool,
    /// Whether the court order or settlement agreement
    /// explicitly identifies the amount AND its purpose as
    /// restitution, remediation, or compliance.
    pub court_order_identifies_amount_and_purpose: bool,
    /// Whether the taxpayer can establish that the payment was
    /// actually made for the identified purpose.
    pub taxpayer_established_payment_for_identified_purpose: bool,
    /// Whether the binding order or agreement was issued on or
    /// before December 22, 2017 (TCJA effective date). Pre-TCJA
    /// orders grandfathered.
    pub pre_december_22_2017_binding_order: bool,
    /// Payment amount in cents (drives § 6050X reporting
    /// threshold).
    pub payment_amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section162fResult {
    pub deductible: bool,
    pub form_1098_f_reporting_required: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section162fInput) -> Section162fResult {
    let mut notes: Vec<String> = Vec::new();

    let form_1098_f_threshold_cents: i64 = 5_000_000;
    let form_1098_f_required = input.payment_amount_cents >= form_1098_f_threshold_cents
        && input.payment_to_government_in_relation_to_violation;

    if input.pre_december_22_2017_binding_order {
        notes.push(
            "TCJA § 13306 — § 162(f) as amended applies only to amounts paid or incurred under orders or agreements after December 22, 2017; pre-TCJA binding orders grandfathered under prior § 162(f) standards"
                .to_string(),
        );
        return Section162fResult {
            deductible: true,
            form_1098_f_reporting_required: form_1098_f_required,
            citation: citation(),
            notes,
        };
    }

    match input.payment_type {
        PaymentType::Fine => {
            if input.payment_to_government_in_relation_to_violation {
                notes.push(
                    "§ 162(f)(1) — fine or civil penalty paid to government in relation to violation of law; NO DEDUCTION allowed"
                        .to_string(),
                );
                if form_1098_f_required {
                    notes.push(
                        "§ 6050X — payment ≥ $50,000 triggers Form 1098-F information reporting by the receiving government entity to the IRS and to the payor"
                            .to_string(),
                    );
                }
                Section162fResult {
                    deductible: false,
                    form_1098_f_reporting_required: form_1098_f_required,
                    citation: citation(),
                    notes,
                }
            } else {
                notes.push(
                    "fine NOT paid to government / in relation to violation — § 162(f) does not apply; standard § 162(a) business expense analysis"
                        .to_string(),
                );
                Section162fResult {
                    deductible: true,
                    form_1098_f_reporting_required: false,
                    citation: citation(),
                    notes,
                }
            }
        }
        PaymentType::RestitutionOrRemediation => {
            check_two_prong_exception(input, &mut notes, "restitution or remediation")
        }
        PaymentType::ComplianceWithLaw => {
            check_two_prong_exception(input, &mut notes, "compliance with law")
        }
        PaymentType::RoutineInvestigationOrCourtCost => {
            notes.push(
                "§ 162(f)(3) — routine investigation costs and court costs do NOT fall within § 162(f)(1) prohibition; standard § 162(a) business expense analysis applies"
                    .to_string(),
            );
            Section162fResult {
                deductible: true,
                form_1098_f_reporting_required: form_1098_f_required,
                citation: citation(),
                notes,
            }
        }
        PaymentType::TaxPayment => {
            notes.push(
                "tax payment to government — § 162(f) does not apply; § 164 governs deductibility of taxes (income tax, real property tax, etc.)"
                    .to_string(),
            );
            Section162fResult {
                deductible: true,
                form_1098_f_reporting_required: false,
                citation: citation(),
                notes,
            }
        }
        PaymentType::QuiTamPayment => {
            notes.push(
                "§ 162(f)(5) — qui tam payment to private relator under False Claims Act or similar statute; relator is NOT government for § 162(f)(1) purposes; standard § 162(a) business expense analysis applies"
                    .to_string(),
            );
            Section162fResult {
                deductible: true,
                form_1098_f_reporting_required: false,
                citation: citation(),
                notes,
            }
        }
    }
}

fn check_two_prong_exception(
    input: &Section162fInput,
    notes: &mut Vec<String>,
    category: &str,
) -> Section162fResult {
    let form_1098_f_threshold_cents: i64 = 5_000_000;
    let form_1098_f_required = input.payment_amount_cents >= form_1098_f_threshold_cents
        && input.payment_to_government_in_relation_to_violation;

    let identification_satisfied = input.court_order_identifies_amount_and_purpose;
    let establishment_satisfied = input.taxpayer_established_payment_for_identified_purpose;

    if !identification_satisfied {
        notes.push(format!(
            "§ 162(f)(2)(B) IDENTIFICATION requirement — court order or settlement agreement must explicitly identify the {} amount and purpose; NO DEDUCTION without identification",
            category
        ));
    }
    if !establishment_satisfied {
        notes.push(format!(
            "§ 162(f)(2)(A) ESTABLISHMENT requirement — taxpayer must establish that the payment was actually made for the identified {} purpose; NO DEDUCTION without establishment",
            category
        ));
    }

    let deductible = identification_satisfied && establishment_satisfied;

    if deductible {
        notes.push(format!(
            "§ 162(f)(2)(A) {} exception engaged — both identification (§ 162(f)(2)(B)) and establishment prongs satisfied; § 162(a) deduction available subject to ordinary business expense analysis",
            category
        ));
    } else {
        notes.push(
            "§ 162(f)(1) general rule applies — no deduction allowed when exception prongs unsatisfied"
                .to_string(),
        );
    }

    if form_1098_f_required {
        notes.push(
            "§ 6050X — payment ≥ $50,000 triggers Form 1098-F information reporting by government to IRS and payor"
                .to_string(),
        );
    }

    Section162fResult {
        deductible,
        form_1098_f_reporting_required: form_1098_f_required,
        citation: citation(),
        notes: notes.clone(),
    }
}

fn citation() -> &'static str {
    "IRC § 162(f)(1)/(2)(A)(i)/(A)(ii)/(B)/(3)/(5); § 6050X; § 162(q); § 164; TCJA 2017 § 13306; Treas. Reg. § 1.162-21 (TD 9946, January 19, 2021)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(payment_type: PaymentType) -> Section162fInput {
        Section162fInput {
            payment_type,
            payment_to_government_in_relation_to_violation: true,
            court_order_identifies_amount_and_purpose: false,
            taxpayer_established_payment_for_identified_purpose: false,
            pre_december_22_2017_binding_order: false,
            payment_amount_cents: 100_000_00,
        }
    }

    #[test]
    fn fine_paid_to_government_nondeductible() {
        let r = compute(&base(PaymentType::Fine));
        assert!(!r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 162(f)(1)") && n.contains("NO DEDUCTION")));
    }

    #[test]
    fn fine_not_to_government_falls_outside_162f() {
        let mut i = base(PaymentType::Fine);
        i.payment_to_government_in_relation_to_violation = false;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("does not apply") || n.contains("§ 162(a)")));
    }

    #[test]
    fn restitution_with_both_prongs_deductible() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 162(f)(2)(A)") && n.contains("restitution")));
    }

    #[test]
    fn restitution_without_identification_nondeductible() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.court_order_identifies_amount_and_purpose = false;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(!r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 162(f)(2)(B) IDENTIFICATION")));
    }

    #[test]
    fn restitution_without_establishment_nondeductible() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = false;
        let r = compute(&i);
        assert!(!r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 162(f)(2)(A) ESTABLISHMENT")));
    }

    #[test]
    fn compliance_with_law_with_both_prongs_deductible() {
        let mut i = base(PaymentType::ComplianceWithLaw);
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r.notes.iter().any(|n| n.contains("compliance with law")));
    }

    #[test]
    fn pre_december_22_2017_binding_order_grandfathered() {
        let mut i = base(PaymentType::Fine);
        i.pre_december_22_2017_binding_order = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("TCJA § 13306") && n.contains("grandfathered")));
    }

    #[test]
    fn routine_investigation_cost_falls_outside_162f1() {
        let r = compute(&base(PaymentType::RoutineInvestigationOrCourtCost));
        assert!(r.deductible);
        assert!(r.notes.iter().any(|n| n.contains("§ 162(f)(3)")));
    }

    #[test]
    fn tax_payment_governed_by_section_164() {
        let r = compute(&base(PaymentType::TaxPayment));
        assert!(r.deductible);
        assert!(r.notes.iter().any(|n| n.contains("§ 164")));
    }

    #[test]
    fn qui_tam_payment_falls_outside_162f1() {
        let r = compute(&base(PaymentType::QuiTamPayment));
        assert!(r.deductible);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 162(f)(5)") && n.contains("relator is NOT government")));
    }

    #[test]
    fn form_1098_f_reporting_required_at_50k_threshold() {
        let mut i = base(PaymentType::Fine);
        i.payment_amount_cents = 50_000_00;
        let r = compute(&i);
        assert!(r.form_1098_f_reporting_required);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6050X") && n.contains("$50,000")));
    }

    #[test]
    fn form_1098_f_not_required_below_50k() {
        let mut i = base(PaymentType::Fine);
        i.payment_amount_cents = 49_999_99;
        let r = compute(&i);
        assert!(!r.form_1098_f_reporting_required);
    }

    #[test]
    fn form_1098_f_at_exact_50k_boundary() {
        let mut i = base(PaymentType::Fine);
        i.payment_amount_cents = 50_000_00;
        let r = compute(&i);
        assert!(r.form_1098_f_reporting_required);
    }

    #[test]
    fn restitution_meeting_both_prongs_deductible_with_form_1098_f() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        i.payment_amount_cents = 100_000_00;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r.form_1098_f_reporting_required);
    }

    #[test]
    fn citation_pins_all_subsections_and_authorities() {
        let r = compute(&base(PaymentType::Fine));
        assert!(r.citation.contains("§ 162(f)(1)"));
        assert!(r.citation.contains("(2)(A)(i)"));
        assert!(r.citation.contains("(A)(ii)"));
        assert!(r.citation.contains("(B)"));
        assert!(r.citation.contains("(3)"));
        assert!(r.citation.contains("(5)"));
        assert!(r.citation.contains("§ 6050X"));
        assert!(r.citation.contains("§ 162(q)"));
        assert!(r.citation.contains("§ 164"));
        assert!(r.citation.contains("TCJA 2017 § 13306"));
        assert!(r.citation.contains("§ 1.162-21"));
        assert!(r.citation.contains("TD 9946"));
        assert!(r.citation.contains("January 19, 2021"));
    }

    #[test]
    fn both_prongs_required_invariant_for_exception_engagement() {
        // Truth table: (identification, establishment, deductible)
        let cases = [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ];
        for (ident, estab, expected) in cases {
            let mut i = base(PaymentType::RestitutionOrRemediation);
            i.court_order_identifies_amount_and_purpose = ident;
            i.taxpayer_established_payment_for_identified_purpose = estab;
            let r = compute(&i);
            assert_eq!(
                r.deductible, expected,
                "ident={} estab={} expected={}",
                ident, estab, expected
            );
        }
    }

    #[test]
    fn pre_tcja_grandfather_bypasses_all_other_prong_checks() {
        let mut i = base(PaymentType::Fine);
        i.pre_december_22_2017_binding_order = true;
        i.court_order_identifies_amount_and_purpose = false;
        i.taxpayer_established_payment_for_identified_purpose = false;
        let r = compute(&i);
        assert!(r.deductible);
    }

    #[test]
    fn nondeductible_fine_still_triggers_1098f_reporting() {
        let mut i = base(PaymentType::Fine);
        i.payment_amount_cents = 500_000_00;
        let r = compute(&i);
        assert!(!r.deductible);
        assert!(r.form_1098_f_reporting_required);
    }

    #[test]
    fn restitution_amount_zero_no_1098f_reporting() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.payment_amount_cents = 0;
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(!r.form_1098_f_reporting_required);
    }

    #[test]
    fn fine_not_in_relation_to_violation_no_1098f() {
        let mut i = base(PaymentType::Fine);
        i.payment_to_government_in_relation_to_violation = false;
        i.payment_amount_cents = 500_000_00;
        let r = compute(&i);
        assert!(!r.form_1098_f_reporting_required);
    }

    #[test]
    fn compliance_with_law_truth_table_invariant() {
        let cases = [
            (false, false, false),
            (false, true, false),
            (true, false, false),
            (true, true, true),
        ];
        for (ident, estab, expected) in cases {
            let mut i = base(PaymentType::ComplianceWithLaw);
            i.court_order_identifies_amount_and_purpose = ident;
            i.taxpayer_established_payment_for_identified_purpose = estab;
            let r = compute(&i);
            assert_eq!(r.deductible, expected);
        }
    }

    #[test]
    fn routine_investigation_cost_invariant_deductible_regardless_of_prongs() {
        let mut i = base(PaymentType::RoutineInvestigationOrCourtCost);
        i.court_order_identifies_amount_and_purpose = false;
        i.taxpayer_established_payment_for_identified_purpose = false;
        let r = compute(&i);
        assert!(r.deductible);
    }

    #[test]
    fn six_payment_type_classification_invariant() {
        let nondeductible_paths = [PaymentType::Fine];
        for pt in nondeductible_paths {
            let r = compute(&base(pt));
            assert!(
                !r.deductible,
                "payment type {:?} should be nondeductible without exception",
                pt
            );
        }

        let deductible_without_exception_paths = [
            PaymentType::RoutineInvestigationOrCourtCost,
            PaymentType::TaxPayment,
            PaymentType::QuiTamPayment,
        ];
        for pt in deductible_without_exception_paths {
            let r = compute(&base(pt));
            assert!(
                r.deductible,
                "payment type {:?} should be deductible regardless",
                pt
            );
        }

        let requires_exception_paths = [
            PaymentType::RestitutionOrRemediation,
            PaymentType::ComplianceWithLaw,
        ];
        for pt in requires_exception_paths {
            let r = compute(&base(pt));
            assert!(
                !r.deductible,
                "payment type {:?} requires both prongs for deduction",
                pt
            );
        }
    }

    #[test]
    fn section_162q_referenced_in_citation_for_sexual_harassment_carve_out() {
        let r = compute(&base(PaymentType::Fine));
        assert!(r.citation.contains("§ 162(q)"));
    }

    #[test]
    fn fine_paid_to_non_governmental_entity_not_in_violation_relation_deductible() {
        let mut i = base(PaymentType::Fine);
        i.payment_to_government_in_relation_to_violation = false;
        let r = compute(&i);
        assert!(r.deductible);
    }

    #[test]
    fn restitution_one_cent_above_50k_triggers_1098f() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.payment_amount_cents = 50_000_00 + 1;
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(r.form_1098_f_reporting_required);
    }

    #[test]
    fn restitution_one_cent_below_50k_no_1098f() {
        let mut i = base(PaymentType::RestitutionOrRemediation);
        i.payment_amount_cents = 50_000_00 - 1;
        i.court_order_identifies_amount_and_purpose = true;
        i.taxpayer_established_payment_for_identified_purpose = true;
        let r = compute(&i);
        assert!(r.deductible);
        assert!(!r.form_1098_f_reporting_required);
    }
}
