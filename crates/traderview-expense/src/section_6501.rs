//! IRC § 6501 — Limitations on assessment and collection (ASED,
//! Assessment Statute Expiration Date).
//!
//! The ASED is the IRS's deadline to assess additional tax. After
//! the ASED passes, the IRS is BARRED from issuing a Notice of
//! Deficiency or assessing additional tax. Critical trader-tax
//! procedural protection — the strongest defensive shield against
//! IRS audit overreach on stale returns. Distinct from § 6502
//! Collection Statute Expiration Date (CSED — 10 years post-
//! assessment to COLLECT) and § 6511 refund SOL (3 years to
//! CLAIM refund).
//!
//! **Seven assessment-period pathways:**
//!
//! **§ 6501(a) — 3-year default**. From filing date for any return
//! filed on or after due date. Most common ASED pathway.
//!
//! **§ 6501(b)(1) — return filed before due date deemed filed on
//! due date**. Early filing does not shorten ASED — clock starts
//! from statutory due date (e.g., April 15 for individual
//! returns). Filed March 1 → ASED still April 15 + 3 years.
//!
//! **§ 6501(e)(1)(A)(i) — 6-year extended for >25% gross-income
//! omission**. If taxpayer omits from gross income more than 25%
//! of gross income stated in return, SOL extended to 6 years.
//! Substantial-understatement defense for IRS.
//!
//! **§ 6501(e)(1)(B) — 6-year for basis overstatement** (post-2015
//! Surface Transportation and Veterans Health Care Choice
//! Improvement Act of 2015 amendment, overruling Home Concrete &
//! Supply v. United States, 132 S. Ct. 1836 (2012)). Basis
//! overstatement now counts as gross-income omission for § 6501(e)
//! 6-year purposes.
//!
//! **§ 6501(c)(1) — UNLIMITED for false or fraudulent return with
//! intent to evade tax**. No SOL — IRS can assess at any time.
//! Highest burden on IRS to prove fraud (clear and convincing).
//!
//! **§ 6501(c)(2) — UNLIMITED for willful attempt to evade tax**.
//! No SOL.
//!
//! **§ 6501(c)(3) — UNLIMITED for no return filed**. No SOL until
//! return is actually filed (then 3-year clock starts).
//!
//! **§ 6501(c)(4) — Form 872 consent extension**. Taxpayer may
//! voluntarily extend ASED via Form 872 (Consent to Extend the
//! Time to Assess Tax). Per IRM 25.6.22, IRS MUST notify taxpayer
//! of three rights: (1) right to refuse extension entirely, (2)
//! right to limit extension to specific issues (Restricted
//! Consent), (3) right to limit extension to specific date.
//! Form 872-A (open-ended consent) terminates upon notice.
//!
//! **Trader-relevant**: Wash-sale loss disallowances, § 1256
//! mark-to-market, § 988 currency, QSBS § 1202 holding periods —
//! all reach back through ASED. Aggressive positions on these
//! sections face fraud (§ 6501(c)(1)) or basis-overstatement
//! (§ 6501(e)(1)(B)) extended SOL risk.
//!
//! Citations: IRC § 6501(a) (3-year default); § 6501(b)(1) (early-
//! filing deemed due-date); § 6501(c)(1) (unlimited fraud);
//! § 6501(c)(2) (unlimited willful evade); § 6501(c)(3) (unlimited
//! no return); § 6501(c)(4) (Form 872 consent); § 6501(e)(1)(A)(i)
//! (6-year >25% omission); § 6501(e)(1)(B) (6-year basis
//! overstatement); IRM 25.6.22 (extension by consent); Home
//! Concrete & Supply, LLC v. United States, 132 S. Ct. 1836 (2012)
//! (overruled by 2015 statutory amendment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AsedPathway {
    /// § 6501(a) — 3-year default from filing date.
    ThreeYearDefault,
    /// § 6501(b)(1) — early filing deemed filed on due date.
    EarlyFilingDeemedDueDate,
    /// § 6501(e)(1)(A)(i) — 6-year for >25% gross-income omission.
    SixYearGrossIncomeOmission,
    /// § 6501(e)(1)(B) — 6-year for basis overstatement (post-2015).
    SixYearBasisOverstatement,
    /// § 6501(c)(1) — unlimited for false/fraudulent return.
    UnlimitedFraud,
    /// § 6501(c)(2) — unlimited for willful attempt to evade.
    UnlimitedWillfulEvade,
    /// § 6501(c)(3) — unlimited for no return filed.
    UnlimitedNoReturnFiled,
    /// § 6501(c)(4) — Form 872 consent extension.
    Form872Consent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6501Input {
    /// Whether a return was filed at all.
    pub return_filed: bool,
    /// Whether the return was filed before the statutory due date.
    pub filed_before_due_date: bool,
    /// Whether the IRS alleges (and could prove) the return is
    /// false or fraudulent with intent to evade tax.
    pub fraudulent_return_intent_to_evade: bool,
    /// Whether the IRS alleges (and could prove) the taxpayer
    /// willfully attempted to evade tax.
    pub willful_attempt_to_evade: bool,
    /// Whether gross income omitted exceeds 25% of gross income
    /// stated in the return (§ 6501(e)(1)(A)(i)).
    pub gross_income_omission_exceeds_25_percent: bool,
    /// Whether the IRS alleges basis overstatement triggering
    /// § 6501(e)(1)(B) post-2015.
    pub basis_overstatement: bool,
    /// Whether the taxpayer signed Form 872 / 872-A consent to
    /// extend ASED.
    pub form_872_signed: bool,
    /// Whether the taxpayer was notified of all three Form 872
    /// rights per IRM 25.6.22 (right to refuse, right to limit
    /// to specific issues, right to limit to specific date).
    pub form_872_three_rights_disclosed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6501Result {
    pub pathway: AsedPathway,
    /// Years from ASED-start date to ASED expiration. None when
    /// unlimited (§ 6501(c)(1)/(c)(2)/(c)(3)) or open-ended
    /// 872 consent.
    pub ased_period_years: Option<u32>,
    pub ased_unlimited: bool,
    pub form_872_three_rights_violation: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6501Input) -> Section6501Result {
    let mut notes: Vec<String> = Vec::new();

    if !input.return_filed {
        notes.push(
            "§ 6501(c)(3) — UNLIMITED ASED until return is actually filed; 3-year clock starts only upon filing"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::UnlimitedNoReturnFiled,
            ased_period_years: None,
            ased_unlimited: true,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::UnlimitedNoReturnFiled),
            notes,
        };
    }

    if input.fraudulent_return_intent_to_evade {
        notes.push(
            "§ 6501(c)(1) — UNLIMITED ASED for false or fraudulent return with intent to evade tax; IRS bears clear-and-convincing burden of proof for fraud"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::UnlimitedFraud,
            ased_period_years: None,
            ased_unlimited: true,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::UnlimitedFraud),
            notes,
        };
    }

    if input.willful_attempt_to_evade {
        notes.push("§ 6501(c)(2) — UNLIMITED ASED for willful attempt to evade tax".to_string());
        return Section6501Result {
            pathway: AsedPathway::UnlimitedWillfulEvade,
            ased_period_years: None,
            ased_unlimited: true,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::UnlimitedWillfulEvade),
            notes,
        };
    }

    if input.form_872_signed {
        let mut three_rights_violation = false;
        if !input.form_872_three_rights_disclosed {
            three_rights_violation = true;
            notes.push(
                "IRM 25.6.22 — IRS MUST notify taxpayer of THREE rights before Form 872 signed: (1) right to refuse extension entirely, (2) right to limit extension to specific issues (Restricted Consent), (3) right to limit extension to specific date"
                    .to_string(),
            );
        }
        notes.push(
            "§ 6501(c)(4) — Form 872 consent extends ASED beyond default; Form 872-A open-ended consent terminates upon written notice"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::Form872Consent,
            ased_period_years: None,
            ased_unlimited: false,
            form_872_three_rights_violation: three_rights_violation,
            citation: citation_for(AsedPathway::Form872Consent),
            notes,
        };
    }

    if input.gross_income_omission_exceeds_25_percent {
        notes.push(
            "§ 6501(e)(1)(A)(i) — 6-year extended SOL for omission of gross income exceeding 25% of gross income stated in return"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::SixYearGrossIncomeOmission,
            ased_period_years: Some(6),
            ased_unlimited: false,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::SixYearGrossIncomeOmission),
            notes,
        };
    }

    if input.basis_overstatement {
        notes.push(
            "§ 6501(e)(1)(B) — 6-year extended SOL for basis overstatement; post-2015 Surface Transportation Act amendment overrules Home Concrete & Supply v. United States, 132 S. Ct. 1836 (2012)"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::SixYearBasisOverstatement,
            ased_period_years: Some(6),
            ased_unlimited: false,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::SixYearBasisOverstatement),
            notes,
        };
    }

    if input.filed_before_due_date {
        notes.push(
            "§ 6501(b)(1) — early-filed return deemed filed on statutory due date; early filing does NOT shorten ASED — clock starts from April 15 (or applicable due date), not actual filing date"
                .to_string(),
        );
        return Section6501Result {
            pathway: AsedPathway::EarlyFilingDeemedDueDate,
            ased_period_years: Some(3),
            ased_unlimited: false,
            form_872_three_rights_violation: false,
            citation: citation_for(AsedPathway::EarlyFilingDeemedDueDate),
            notes,
        };
    }

    notes.push("§ 6501(a) — 3-year default ASED from filing date; most common pathway".to_string());
    Section6501Result {
        pathway: AsedPathway::ThreeYearDefault,
        ased_period_years: Some(3),
        ased_unlimited: false,
        form_872_three_rights_violation: false,
        citation: citation_for(AsedPathway::ThreeYearDefault),
        notes,
    }
}

fn citation_for(pathway: AsedPathway) -> &'static str {
    match pathway {
        AsedPathway::ThreeYearDefault => "IRC § 6501(a)",
        AsedPathway::EarlyFilingDeemedDueDate => "IRC §§ 6501(a), 6501(b)(1)",
        AsedPathway::SixYearGrossIncomeOmission => "IRC § 6501(e)(1)(A)(i)",
        AsedPathway::SixYearBasisOverstatement => "IRC § 6501(e)(1)(B); Home Concrete & Supply v. United States, 132 S. Ct. 1836 (2012) overruled by Surface Transportation Act of 2015",
        AsedPathway::UnlimitedFraud => "IRC § 6501(c)(1)",
        AsedPathway::UnlimitedWillfulEvade => "IRC § 6501(c)(2)",
        AsedPathway::UnlimitedNoReturnFiled => "IRC § 6501(c)(3)",
        AsedPathway::Form872Consent => "IRC § 6501(c)(4); IRM 25.6.22",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6501Input {
        Section6501Input {
            return_filed: true,
            filed_before_due_date: false,
            fraudulent_return_intent_to_evade: false,
            willful_attempt_to_evade: false,
            gross_income_omission_exceeds_25_percent: false,
            basis_overstatement: false,
            form_872_signed: false,
            form_872_three_rights_disclosed: false,
        }
    }

    #[test]
    fn three_year_default_pathway() {
        let r = check(&base());
        assert_eq!(r.pathway, AsedPathway::ThreeYearDefault);
        assert_eq!(r.ased_period_years, Some(3));
        assert!(!r.ased_unlimited);
    }

    #[test]
    fn three_year_default_citation_pins_section_a() {
        let r = check(&base());
        assert_eq!(r.citation, "IRC § 6501(a)");
    }

    #[test]
    fn three_year_default_note_describes_pathway() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(a)") && n.contains("3-year default")));
    }

    #[test]
    fn no_return_filed_unlimited() {
        let mut i = base();
        i.return_filed = false;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedNoReturnFiled);
        assert!(r.ased_unlimited);
        assert_eq!(r.ased_period_years, None);
        assert!(r.citation.contains("§ 6501(c)(3)"));
    }

    #[test]
    fn no_return_filed_note_describes_clock_start() {
        let mut i = base();
        i.return_filed = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(3)")
                && n.contains("3-year clock starts only upon filing")));
    }

    #[test]
    fn fraud_intent_to_evade_unlimited() {
        let mut i = base();
        i.fraudulent_return_intent_to_evade = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedFraud);
        assert!(r.ased_unlimited);
        assert_eq!(r.citation, "IRC § 6501(c)(1)");
    }

    #[test]
    fn fraud_note_describes_clear_and_convincing_burden() {
        let mut i = base();
        i.fraudulent_return_intent_to_evade = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(1)") && n.contains("clear-and-convincing")));
    }

    #[test]
    fn willful_evade_unlimited() {
        let mut i = base();
        i.willful_attempt_to_evade = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedWillfulEvade);
        assert!(r.ased_unlimited);
        assert_eq!(r.citation, "IRC § 6501(c)(2)");
    }

    #[test]
    fn six_year_gross_income_omission_pathway() {
        let mut i = base();
        i.gross_income_omission_exceeds_25_percent = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::SixYearGrossIncomeOmission);
        assert_eq!(r.ased_period_years, Some(6));
        assert!(!r.ased_unlimited);
        assert_eq!(r.citation, "IRC § 6501(e)(1)(A)(i)");
    }

    #[test]
    fn six_year_basis_overstatement_pathway() {
        let mut i = base();
        i.basis_overstatement = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::SixYearBasisOverstatement);
        assert_eq!(r.ased_period_years, Some(6));
        assert!(!r.ased_unlimited);
    }

    #[test]
    fn six_year_basis_citation_pins_home_concrete_and_2015_amendment() {
        let mut i = base();
        i.basis_overstatement = true;
        let r = check(&i);
        assert!(r.citation.contains("§ 6501(e)(1)(B)"));
        assert!(r.citation.contains("Home Concrete"));
        assert!(r.citation.contains("Surface Transportation Act of 2015"));
    }

    #[test]
    fn six_year_basis_note_describes_amendment_overrule() {
        let mut i = base();
        i.basis_overstatement = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(e)(1)(B)") && n.contains("overrules Home Concrete")));
    }

    #[test]
    fn early_filing_deemed_due_date_pathway() {
        let mut i = base();
        i.filed_before_due_date = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::EarlyFilingDeemedDueDate);
        assert_eq!(r.ased_period_years, Some(3));
    }

    #[test]
    fn early_filing_note_describes_due_date_anchor() {
        let mut i = base();
        i.filed_before_due_date = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(b)(1)") && n.contains("early filing does NOT shorten")));
    }

    #[test]
    fn form_872_signed_pathway() {
        let mut i = base();
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::Form872Consent);
        assert!(!r.form_872_three_rights_violation);
    }

    #[test]
    fn form_872_without_three_rights_disclosure_violation() {
        let mut i = base();
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = false;
        let r = check(&i);
        assert!(r.form_872_three_rights_violation);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("IRM 25.6.22") && n.contains("THREE rights")));
    }

    #[test]
    fn form_872_a_open_ended_note() {
        let mut i = base();
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(4)") && n.contains("Form 872-A open-ended")));
    }

    #[test]
    fn form_872_citation_pins_section_c_4_and_irm() {
        let mut i = base();
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = true;
        let r = check(&i);
        assert!(r.citation.contains("§ 6501(c)(4)"));
        assert!(r.citation.contains("IRM 25.6.22"));
    }

    #[test]
    fn priority_order_fraud_beats_six_year_omission() {
        let mut i = base();
        i.gross_income_omission_exceeds_25_percent = true;
        i.fraudulent_return_intent_to_evade = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedFraud);
    }

    #[test]
    fn priority_order_no_return_beats_fraud() {
        let mut i = base();
        i.return_filed = false;
        i.fraudulent_return_intent_to_evade = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedNoReturnFiled);
    }

    #[test]
    fn priority_order_fraud_beats_willful() {
        let mut i = base();
        i.fraudulent_return_intent_to_evade = true;
        i.willful_attempt_to_evade = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedFraud);
    }

    #[test]
    fn priority_order_six_year_omission_beats_six_year_basis() {
        let mut i = base();
        i.gross_income_omission_exceeds_25_percent = true;
        i.basis_overstatement = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::SixYearGrossIncomeOmission);
    }

    #[test]
    fn priority_order_form_872_beats_six_year() {
        let mut i = base();
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = true;
        i.gross_income_omission_exceeds_25_percent = true;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::Form872Consent);
    }

    #[test]
    fn unlimited_pathways_have_no_period_years() {
        for pathway_input in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let i = Section6501Input {
                return_filed: !pathway_input.2,
                filed_before_due_date: false,
                fraudulent_return_intent_to_evade: pathway_input.0,
                willful_attempt_to_evade: pathway_input.1,
                gross_income_omission_exceeds_25_percent: false,
                basis_overstatement: false,
                form_872_signed: false,
                form_872_three_rights_disclosed: false,
            };
            let r = check(&i);
            assert_eq!(r.ased_period_years, None);
            assert!(r.ased_unlimited);
        }
    }

    #[test]
    fn six_year_pathways_have_six_year_period() {
        let mut i_omission = base();
        i_omission.gross_income_omission_exceeds_25_percent = true;
        assert_eq!(check(&i_omission).ased_period_years, Some(6));

        let mut i_basis = base();
        i_basis.basis_overstatement = true;
        assert_eq!(check(&i_basis).ased_period_years, Some(6));
    }

    #[test]
    fn three_year_pathways_have_three_year_period() {
        let r_default = check(&base());
        assert_eq!(r_default.ased_period_years, Some(3));

        let mut i_early = base();
        i_early.filed_before_due_date = true;
        assert_eq!(check(&i_early).ased_period_years, Some(3));
    }

    #[test]
    fn pathway_routing_eight_cells_invariant() {
        let pathways = [
            (
                Section6501Input {
                    return_filed: false,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::UnlimitedNoReturnFiled,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: true,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::UnlimitedFraud,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: true,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::UnlimitedWillfulEvade,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: true,
                    form_872_three_rights_disclosed: true,
                },
                AsedPathway::Form872Consent,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: true,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::SixYearGrossIncomeOmission,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: true,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::SixYearBasisOverstatement,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: true,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::EarlyFilingDeemedDueDate,
            ),
            (
                Section6501Input {
                    return_filed: true,
                    filed_before_due_date: false,
                    fraudulent_return_intent_to_evade: false,
                    willful_attempt_to_evade: false,
                    gross_income_omission_exceeds_25_percent: false,
                    basis_overstatement: false,
                    form_872_signed: false,
                    form_872_three_rights_disclosed: false,
                },
                AsedPathway::ThreeYearDefault,
            ),
        ];

        for (input, expected) in pathways {
            assert_eq!(check(&input).pathway, expected);
        }
    }

    #[test]
    fn three_rights_violation_only_when_form_872_signed() {
        let mut i = base();
        i.form_872_three_rights_disclosed = false;
        let r = check(&i);
        assert!(!r.form_872_three_rights_violation);
    }

    #[test]
    fn three_rights_violation_routed_via_form_872_pathway_only() {
        let mut i = base();
        i.fraudulent_return_intent_to_evade = true;
        i.form_872_signed = true;
        i.form_872_three_rights_disclosed = false;
        let r = check(&i);
        assert_eq!(r.pathway, AsedPathway::UnlimitedFraud);
        assert!(!r.form_872_three_rights_violation);
    }

    #[test]
    fn unlimited_pathways_all_set_unlimited_flag() {
        for (return_filed, fraud, willful) in [
            (false, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            let i = Section6501Input {
                return_filed,
                filed_before_due_date: false,
                fraudulent_return_intent_to_evade: fraud,
                willful_attempt_to_evade: willful,
                gross_income_omission_exceeds_25_percent: false,
                basis_overstatement: false,
                form_872_signed: false,
                form_872_three_rights_disclosed: false,
            };
            assert!(check(&i).ased_unlimited);
        }
    }

    #[test]
    fn non_unlimited_pathways_do_not_set_unlimited_flag() {
        let mut i = base();
        i.gross_income_omission_exceeds_25_percent = true;
        assert!(!check(&i).ased_unlimited);

        let mut i_basis = base();
        i_basis.basis_overstatement = true;
        assert!(!check(&i_basis).ased_unlimited);

        let mut i_early = base();
        i_early.filed_before_due_date = true;
        assert!(!check(&i_early).ased_unlimited);
    }
}
