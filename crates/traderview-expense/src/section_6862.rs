//! IRC § 6862 — Jeopardy assessment of taxes other than
//! income, estate, gift, and certain excise taxes. Completes
//! the jeopardy/termination cluster — § 6851 (income tax
//! termination), § 6861 (income/estate/gift jeopardy), §
//! 6862 (other-tax jeopardy), § 6863 (stay of collection),
//! § 7429 (review). Trader-procedural-critical for traders
//! operating businesses with employment tax obligations
//! (§ 3402 income tax withholding + § 3111 FICA) and excise
//! tax obligations (§ 5000A ACA, § 4980H employer-mandate,
//! § 4940-4945 private-foundation chapter 42 taxes) when
//! Secretary believes collection of such taxes will be
//! jeopardized by delay. Companion to `section_6861`
//! (jeopardy income/estate/gift — already shipped),
//! `section_6851` (termination assessment income tax —
//! already shipped), `section_6863` (stay of collection —
//! already shipped), `section_6213` (Tax Court petition
//! window), `section_6321` (lien), `section_6331` (levy),
//! § 7429 (review of jeopardy procedures).
//!
//! **§ 6862(a) Authority** — if Secretary believes that
//! collection of any tax (other than income tax, estate
//! tax, gift tax, and the excise taxes imposed by chapters
//! **41, 42, 43, and 44**) under any provision of the
//! internal revenue laws will be **jeopardized by delay**,
//! Secretary shall, **whether or not the time otherwise
//! prescribed by law for making return and paying such tax
//! has expired**, immediately assess such tax (together
//! with all interest, additional amounts, and additions to
//! the tax provided for by law). Such tax, additions to
//! tax, and interest shall thereupon become **immediately
//! due and payable**, and immediate notice and demand shall
//! be made by the Secretary for the payment thereof.
//!
//! **§ 6862(b) Immediate levy** — collection by levy under
//! § 6331(a) is authorized **without regard to the 10-day
//! notice requirement** that would otherwise apply after
//! notice and demand.
//!
//! **Chapter 41-44 carve-outs** (covered by § 6861, NOT
//! § 6862):
//! - Chapter 41 — Public Charities (private foundation
//!   tax)
//! - Chapter 42 — Private Foundations and Certain Other
//!   Tax-Exempt Organizations (§§ 4940-4948 excise taxes)
//! - Chapter 43 — Qualified Pension, etc., Plans (§§
//!   4971-4980 excise taxes)
//! - Chapter 44 — Real Estate Investment Trusts (§§ 4981-
//!   4982)
//!
//! **Taxes within § 6862 scope** include:
//! - **Employment taxes** — § 3402 (income tax
//!   withholding); § 3111 (employer FICA + Medicare); §
//!   3301 (FUTA); § 3406 (backup withholding)
//! - **Excise taxes** (NOT in chapters 41-44) — § 5001
//!   (alcohol); § 5701 (tobacco); § 4081 (fuel); § 4221
//!   (manufacturer); § 4251 (communications); § 4261 (air
//!   transportation)
//! - **Foreign withholding taxes** — § 1441-1446 chapters 3
//!   and 4 FATCA
//! - **Trust fund recovery penalty** (§ 6672) — but § 6672
//!   itself has separate procedures
//!
//! **§ 7429 review procedures** (cross-referenced):
//! - § 7429(a)(1)(A) **Chief Counsel for IRS personal
//!   written approval REQUIRED** before § 6862 jeopardy
//!   assessment or § 6331 jeopardy levy.
//! - § 7429(a)(1)(B) **written statement of information
//!   relied upon within 5 days** of jeopardy assessment;
//!   must state SPECIFIC FACTS and REASONS (not mere
//!   conclusions); conclusion-only notice may invalidate
//!   assessment.
//! - § 7429(a)(2) **30-day administrative review** window
//!   for taxpayer.
//! - § 7429(b)(1) **90-day judicial review** in district
//!   court.
//! - § 7429(g)(1) **burden of proof on Secretary** for
//!   reasonableness of jeopardy assessment.
//! - § 7429(g)(2) **burden of proof on taxpayer** for
//!   amount appropriateness.
//!
//! **§ 6862 vs § 6861 distinction**:
//! - **§ 6862** applies to **employment + excise (non-
//!   chapter-41-44) + other** taxes; no 60-day SNOD
//!   requirement (no SNOD applies to most non-deficiency
//!   taxes).
//! - **§ 6861** applies to **income/estate/gift +
//!   chapter-41-44 excise** taxes; § 6212 SNOD framework
//!   applies; § 6861(b) 60-day SNOD requirement.
//!
//! **§ 6862 + § 6863 interaction** — amount of § 6862
//! jeopardy assessment may be **stayed via § 6863 bond**
//! filing; § 6863(b)(3)(A) sale-prohibition on seized
//! property pending § 7429(b) civil action **specifically
//! applies to § 6862(a)** (not § 6861).
//!
//! Citations: 26 USC § 6862(a)-(b); 26 CFR § 301.6862-1;
//! § 6861 (jeopardy income/estate/gift); § 6851
//! (termination income tax); § 6863 (stay of collection);
//! § 6213(a) (Tax Court petition); § 6321 (lien); § 6331
//! (levy); § 7429 (review); IRM 4.15.1 (Jeopardy and
//! Terminations); IRM 5.17.15 (Termination and Jeopardy
//! Assessments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxCategory {
    /// Employment tax (§ 3402 withholding, § 3111 FICA,
    /// § 3301 FUTA, § 3406 backup withholding).
    EmploymentTax,
    /// Excise tax NOT in chapters 41-44 (alcohol § 5001,
    /// tobacco § 5701, fuel § 4081, manufacturer § 4221,
    /// communications § 4251, air transportation § 4261).
    ExciseNonChapter4144,
    /// Foreign withholding tax (§§ 1441-1446 + FATCA
    /// chapter 4).
    ForeignWithholding,
    /// Income tax (excluded — covered by § 6861).
    IncomeTax,
    /// Estate tax (excluded — covered by § 6861).
    EstateTax,
    /// Gift tax (excluded — covered by § 6861).
    GiftTax,
    /// Chapter 41 excise — private foundation (excluded —
    /// covered by § 6861).
    Chapter41PrivateFoundation,
    /// Chapter 42 excise — § 4940-4948 (excluded — covered
    /// by § 6861).
    Chapter42Excise,
    /// Chapter 43 excise — qualified plans § 4971-4980
    /// (excluded — covered by § 6861).
    Chapter43Excise,
    /// Chapter 44 excise — REITs § 4981-4982 (excluded —
    /// covered by § 6861).
    Chapter44Excise,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6862Input {
    pub tax_category: TaxCategory,
    /// Whether Secretary believes collection will be
    /// jeopardized by delay.
    pub secretary_believes_jeopardized: bool,
    /// Whether due date for making return and paying tax
    /// has expired (§ 6862(a) "whether or not" clause —
    /// applies regardless).
    pub due_date_expired: bool,
    /// Whether Chief Counsel for IRS provided personal
    /// written approval (§ 7429(a)(1)(A)).
    pub chief_counsel_personal_approval: bool,
    /// Whether written statement of information was provided
    /// within 5 days (§ 7429(a)(1)(B)).
    pub written_statement_within_5_days: bool,
    /// Whether written statement states SPECIFIC FACTS AND
    /// REASONS (not mere conclusions).
    pub statement_specific_facts_not_conclusions: bool,
    /// Whether § 6863 bond was filed to stay collection.
    pub section_6863_bond_filed: bool,
    /// Whether immediate § 6331 levy was made without 10-
    /// day notice (§ 6862(b) authority).
    pub immediate_levy_without_10_day_notice: bool,
    /// Whether court determined § 7429(g)(1) Secretary
    /// reasonableness burden sustained.
    pub secretary_sustained_reasonableness_burden: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6862Result {
    pub section_6862_applies: bool,
    pub jeopardy_assessment_authorized: bool,
    pub chief_counsel_approval_compliant: bool,
    pub written_statement_compliant: bool,
    pub immediate_levy_authorized: bool,
    pub stay_engaged_via_6863_bond: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6862Input) -> Section6862Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let in_scope = matches!(
        input.tax_category,
        TaxCategory::EmploymentTax
            | TaxCategory::ExciseNonChapter4144
            | TaxCategory::ForeignWithholding
    );

    if !in_scope {
        failure_reasons.push(
            "26 USC § 6862(a) — § 6862 does NOT apply to income tax, estate tax, gift tax, or chapter 41/42/43/44 excise taxes; § 6861 jeopardy assessment framework applies instead".to_string(),
        );
    }

    if !input.secretary_believes_jeopardized {
        failure_reasons.push(
            "26 USC § 6862(a) — Secretary must believe collection of tax will be JEOPARDIZED BY DELAY".to_string(),
        );
    }

    if !input.chief_counsel_personal_approval {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(A) — Chief Counsel for IRS personal written approval REQUIRED before § 6862 jeopardy assessment or § 6331 jeopardy levy".to_string(),
        );
    }

    if !input.written_statement_within_5_days {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(B) — Secretary shall provide taxpayer written statement of information relied upon within 5 DAYS of jeopardy assessment".to_string(),
        );
    }

    if input.written_statement_within_5_days && !input.statement_specific_facts_not_conclusions {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(B) — written statement must state SPECIFIC FACTS AND REASONS (not mere conclusions); conclusion-only notice may invalidate assessment".to_string(),
        );
    }

    let authorized = in_scope
        && input.secretary_believes_jeopardized
        && input.chief_counsel_personal_approval
        && input.written_statement_within_5_days
        && input.statement_specific_facts_not_conclusions;

    let notes: Vec<String> = vec![
        "26 USC § 6862(a) — if Secretary believes collection of any tax (OTHER than income tax + estate tax + gift tax + chapter 41/42/43/44 excise taxes) will be jeopardized by delay, Secretary shall immediately assess tax together with interest, additional amounts, additions to tax; whether or not due date for making return and paying tax has expired".to_string(),
        "26 USC § 6862(a) — assessed tax becomes IMMEDIATELY DUE AND PAYABLE; immediate notice and demand made by Secretary for payment".to_string(),
        "26 USC § 6862(b) — collection by § 6331(a) levy AUTHORIZED without regard to 10-day notice requirement".to_string(),
        "Chapter 41-44 carve-outs (covered by § 6861, NOT § 6862): Chapter 41 (public charities); Chapter 42 (private foundations §§ 4940-4948); Chapter 43 (qualified plans §§ 4971-4980); Chapter 44 (REITs §§ 4981-4982)".to_string(),
        "§ 6862 in-scope taxes: EMPLOYMENT (§ 3402 withholding, § 3111 FICA, § 3301 FUTA, § 3406 backup withholding) + EXCISE not in chapters 41-44 (alcohol § 5001, tobacco § 5701, fuel § 4081, manufacturer § 4221, communications § 4251, air transportation § 4261) + FOREIGN WITHHOLDING (§§ 1441-1446 + FATCA chapter 4)".to_string(),
        "26 USC § 7429(a)(1)(A) — Chief Counsel for IRS personal written approval REQUIRED before jeopardy assessment or jeopardy levy".to_string(),
        "26 USC § 7429(a)(1)(B) — Secretary must provide written statement within 5 DAYS stating SPECIFIC FACTS AND REASONS (not mere conclusions); conclusion-only notice may invalidate assessment".to_string(),
        "26 USC § 7429(a)(2) + § 7429(b)(1) — taxpayer may request 30-day administrative review + 90-day judicial review in district court".to_string(),
        "§ 6862 vs § 6861 distinction: § 6862 applies to employment + excise (non-chapter-41-44) + foreign withholding taxes; no 60-day SNOD requirement (no SNOD applies to most non-deficiency taxes); § 6861 applies to income/estate/gift + chapter-41-44 excise taxes with § 6212 SNOD framework".to_string(),
        "§ 6862 + § 6863 interaction — jeopardy assessment under § 6862(a) may be stayed via § 6863 bond filing; § 6863(b)(3)(A) sale-prohibition on seized property pending § 7429(b) civil action SPECIFICALLY applies to § 6862(a) (not § 6861)".to_string(),
    ];

    Section6862Result {
        section_6862_applies: in_scope,
        jeopardy_assessment_authorized: authorized,
        chief_counsel_approval_compliant: input.chief_counsel_personal_approval,
        written_statement_compliant: input.written_statement_within_5_days
            && input.statement_specific_facts_not_conclusions,
        immediate_levy_authorized: authorized && input.immediate_levy_without_10_day_notice,
        stay_engaged_via_6863_bond: input.section_6863_bond_filed,
        failure_reasons,
        citation: "26 USC § 6862(a)-(b); 26 CFR § 301.6862-1; § 6861; § 6851; § 6863; § 6213(a); § 6321; § 6331; § 7429; IRM 4.15.1; IRM 5.17.15",
        notes,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn valid_base() -> Section6862Input {
        Section6862Input {
            tax_category: TaxCategory::EmploymentTax,
            secretary_believes_jeopardized: true,
            due_date_expired: true,
            chief_counsel_personal_approval: true,
            written_statement_within_5_days: true,
            statement_specific_facts_not_conclusions: true,
            section_6863_bond_filed: false,
            immediate_levy_without_10_day_notice: false,
            secretary_sustained_reasonableness_burden: true,
        }
    }

    #[test]
    fn employment_tax_in_scope_authorized() {
        let r = check(&valid_base());
        assert!(r.section_6862_applies);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn excise_non_chapter_4144_in_scope_authorized() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::ExciseNonChapter4144;
        let r = check(&i);
        assert!(r.section_6862_applies);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn foreign_withholding_in_scope_authorized() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::ForeignWithholding;
        let r = check(&i);
        assert!(r.section_6862_applies);
    }

    #[test]
    fn income_tax_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::IncomeTax;
        let r = check(&i);
        assert!(!r.section_6862_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6862") && f.contains("§ 6861")));
    }

    #[test]
    fn estate_tax_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::EstateTax;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn gift_tax_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::GiftTax;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn chapter_41_excise_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::Chapter41PrivateFoundation;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn chapter_42_excise_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::Chapter42Excise;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn chapter_43_excise_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::Chapter43Excise;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn chapter_44_excise_out_of_6862_scope() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::Chapter44Excise;
        let r = check(&i);
        assert!(!r.section_6862_applies);
    }

    #[test]
    fn no_secretary_belief_fails() {
        let mut i = valid_base();
        i.secretary_believes_jeopardized = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6862(a)") && f.contains("JEOPARDIZED BY DELAY")));
    }

    #[test]
    fn no_chief_counsel_approval_fails() {
        let mut i = valid_base();
        i.chief_counsel_personal_approval = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(!r.chief_counsel_approval_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(a)(1)(A)") && f.contains("Chief Counsel")));
    }

    #[test]
    fn no_written_statement_fails() {
        let mut i = valid_base();
        i.written_statement_within_5_days = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(!r.written_statement_compliant);
    }

    #[test]
    fn conclusion_only_statement_fails() {
        let mut i = valid_base();
        i.statement_specific_facts_not_conclusions = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert!(!r.written_statement_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("SPECIFIC FACTS AND REASONS")));
    }

    #[test]
    fn due_date_not_expired_still_authorized() {
        let mut i = valid_base();
        i.due_date_expired = false;
        let r = check(&i);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn immediate_levy_authorized_when_compliant() {
        let mut i = valid_base();
        i.immediate_levy_without_10_day_notice = true;
        let r = check(&i);
        assert!(r.immediate_levy_authorized);
    }

    #[test]
    fn immediate_levy_blocked_when_not_authorized() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::IncomeTax;
        i.immediate_levy_without_10_day_notice = true;
        let r = check(&i);
        assert!(!r.immediate_levy_authorized);
    }

    #[test]
    fn section_6863_bond_filed_engages_stay() {
        let mut i = valid_base();
        i.section_6863_bond_filed = true;
        let r = check(&i);
        assert!(r.stay_engaged_via_6863_bond);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6862(a)-(b)"));
        assert!(r.citation.contains("26 CFR § 301.6862-1"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6851"));
        assert!(r.citation.contains("§ 6863"));
        assert!(r.citation.contains("§ 6213(a)"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 7429"));
        assert!(r.citation.contains("IRM 4.15.1"));
        assert!(r.citation.contains("IRM 5.17.15"));
    }

    #[test]
    fn note_pins_section_a_immediate_assessment_authority() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6862(a)")
            && n.contains("jeopardized by delay")
            && n.contains("whether or not due date")));
    }

    #[test]
    fn note_pins_section_a_immediately_due_and_payable() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("IMMEDIATELY DUE AND PAYABLE")));
    }

    #[test]
    fn note_pins_section_b_immediate_levy() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6862(b)")
            && n.contains("§ 6331(a) levy")
            && n.contains("10-day notice")));
    }

    #[test]
    fn note_pins_chapter_4144_carveouts() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Chapter 41")
            && n.contains("Chapter 42")
            && n.contains("Chapter 43")
            && n.contains("Chapter 44")
            && n.contains("§ 6861")));
    }

    #[test]
    fn note_pins_in_scope_taxes_employment_excise_foreign() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("EMPLOYMENT")
            && n.contains("§ 3402")
            && n.contains("§ 3111 FICA")
            && n.contains("§ 3301 FUTA")
            && n.contains("§ 5001")
            && n.contains("FATCA chapter 4")));
    }

    #[test]
    fn note_pins_7429_a1A_chief_counsel_approval() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(a)(1)(A)")
            && n.contains("Chief Counsel")
            && n.contains("personal written approval")));
    }

    #[test]
    fn note_pins_7429_a1B_5_day_specific_facts() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(a)(1)(B)")
            && n.contains("5 DAYS")
            && n.contains("SPECIFIC FACTS AND REASONS")));
    }

    #[test]
    fn note_pins_30_day_90_day_review_windows() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(a)(2)")
            && n.contains("§ 7429(b)(1)")
            && n.contains("30-day")
            && n.contains("90-day")));
    }

    #[test]
    fn note_pins_6862_vs_6861_distinction() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6862 vs § 6861")
            && n.contains("60-day SNOD")
            && n.contains("§ 6212 SNOD framework")));
    }

    #[test]
    fn note_pins_6862_6863_b3A_sale_prohibition() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6862 + § 6863")
            && n.contains("§ 6863(b)(3)(A)")
            && n.contains("SPECIFICALLY applies to § 6862(a)")));
    }

    #[test]
    fn tax_category_truth_table_ten_cells() {
        for (category, exp_in_scope) in [
            (TaxCategory::EmploymentTax, true),
            (TaxCategory::ExciseNonChapter4144, true),
            (TaxCategory::ForeignWithholding, true),
            (TaxCategory::IncomeTax, false),
            (TaxCategory::EstateTax, false),
            (TaxCategory::GiftTax, false),
            (TaxCategory::Chapter41PrivateFoundation, false),
            (TaxCategory::Chapter42Excise, false),
            (TaxCategory::Chapter43Excise, false),
            (TaxCategory::Chapter44Excise, false),
        ] {
            let mut i = valid_base();
            i.tax_category = category;
            let r = check(&i);
            assert_eq!(
                r.section_6862_applies, exp_in_scope,
                "category={:?} expected in_scope={}",
                category, exp_in_scope
            );
        }
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::IncomeTax;
        i.secretary_believes_jeopardized = false;
        i.chief_counsel_personal_approval = false;
        i.written_statement_within_5_days = false;
        let r = check(&i);
        assert!(!r.jeopardy_assessment_authorized);
        assert_eq!(r.failure_reasons.len(), 4);
    }

    #[test]
    fn employment_tax_due_date_not_expired_still_authorized_invariant() {
        let mut i = valid_base();
        i.tax_category = TaxCategory::EmploymentTax;
        i.due_date_expired = false;
        let r = check(&i);
        assert!(r.jeopardy_assessment_authorized);
    }

    #[test]
    fn chapter_41_44_uniquely_handled_by_6861_invariant() {
        for chapter in [
            TaxCategory::Chapter41PrivateFoundation,
            TaxCategory::Chapter42Excise,
            TaxCategory::Chapter43Excise,
            TaxCategory::Chapter44Excise,
        ] {
            let mut i = valid_base();
            i.tax_category = chapter;
            let r = check(&i);
            assert!(!r.section_6862_applies);
            assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6861")));
        }
    }
}
