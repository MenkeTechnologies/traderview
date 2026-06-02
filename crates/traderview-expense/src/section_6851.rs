//! IRC § 6851 — Termination assessments of income tax. The
//! emergency procedure by which IRS may **TERMINATE a
//! taxpayer's taxable year mid-year** when Secretary finds
//! taxpayer designing to depart from US, conceal property,
//! or jeopardize collection. Trader-procedural-critical
//! because § 6851 is the most aggressive form of pre-due-
//! date assessment authority — used against active traders
//! suspected of fleeing US, hiding offshore assets, or
//! dissipating brokerage accounts before tax due date.
//! Companion to `section_6861` (jeopardy assessment income/
//! estate/gift — already shipped), § 6852 (termination
//! assessment qualified person), `section_6863` (stay of
//! collection — already shipped), `section_6213` (Tax Court
//! petition window), `section_6321` (lien), `section_6331`
//! (levy), § 7429 (review of jeopardy procedures).
//!
//! **§ 6851(a)(1) Termination authority** — if Secretary
//! finds that taxpayer designs quickly to:
//! 1. **Depart from the United States** or to remove his
//!    property therefrom;
//! 2. To **conceal himself or his property** therein; OR
//! 3. **To do any other act** (including in the case of a
//!    corporation, distributing all or part of its assets
//!    in liquidation or otherwise) tending to **prejudice
//!    or render wholly or partly ineffectual proceedings
//!    to collect the income tax** for the current or
//!    immediately preceding taxable year unless such
//!    proceedings are brought without delay,
//!
//! the Secretary shall **immediately declare the taxable
//! period for such taxpayer immediately terminated** and
//! shall cause notice of such finding and declaration to be
//! given the taxpayer, together with a demand for immediate
//! payment of the tax for the taxable period so declared
//! terminated and of the tax for the preceding taxable year
//! or so much of such tax as is unpaid.
//!
//! **§ 6851(a)(2) Computation of tax** — the tax for any
//! period terminated under § 6851(a) shall be computed:
//! 1. As if such period were the taxable year of the
//!    taxpayer; AND
//! 2. By placing the entire tax base on an annual basis
//!    (i.e., annualize income for the partial year).
//!
//! **§ 6851(b) Notice of deficiency** — **SNOD must be
//! issued within 60 days** after the LATER of:
//! 1. The **due date of the return for the full taxable
//!    year** (including extensions); OR
//! 2. The **date taxpayer files such return** for the
//!    taxable year.
//!
//! **§ 6851(c) Treatment of amounts collected** — any
//! amount collected as a result of any termination
//! assessment shall, on the making of the assessment of the
//! tax for the entire taxable year, be treated as if such
//! amount had been collected on the date such assessment
//! were made.
//!
//! **§ 6851(d) Cross references**:
//! - § 7429 review of jeopardy assessments + termination
//!   assessments (30-day administrative + 90-day judicial).
//! - § 6863 stay of collection by bond.
//! - § 6213(a) Tax Court petition right preserved by § 6851
//!   60-day SNOD requirement.
//!
//! **§ 6851 vs § 6861 distinction**:
//! - **§ 6851** — termination of CURRENT or immediately
//!   preceding TAXABLE YEAR (income tax only); used
//!   BEFORE return due date.
//! - **§ 6861** — jeopardy assessment of EXISTING
//!   DEFICIENCY (income/estate/gift/certain excise); used
//!   AFTER return filing or SNOD.
//!
//! **§ 6851 + § 6863 interaction**: amount of termination
//! assessment must be paid within **10 days** unless a bond
//! is filed under § 6863 to stay collection.
//!
//! Citations: 26 USC § 6851(a)-(d); 26 CFR § 1.6851-1; § 6852
//! (termination assessment in cases of qualified person);
//! § 6861 (jeopardy assessment); § 6863 (stay of collection);
//! § 6213(a) (Tax Court petition); § 6321 (lien); § 6331
//! (levy); § 7429 (review of jeopardy procedures); IRM
//! 4.15.1 (Jeopardy and Terminations); IRM 5.1.4 (Jeopardy,
//! Termination, Quick and Prompt Assessments); IRM 5.17.15
//! (Termination and Jeopardy Assessments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminationTrigger {
    /// § 6851(a)(1)(A) — taxpayer designs to depart from US
    /// or remove property from US.
    DepartingOrRemovingProperty,
    /// § 6851(a)(1)(B) — taxpayer designs to conceal himself
    /// or his property in US.
    ConcealingSelfOrProperty,
    /// § 6851(a)(1)(C) — taxpayer designs to do any other
    /// act (including corporate liquidation) tending to
    /// prejudice or render ineffectual collection.
    OtherJeopardizingActOrLiquidation,
    /// No § 6851 trigger established.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6851Input {
    pub termination_trigger: TerminationTrigger,
    /// Whether taxable period was annualized for partial
    /// year computation (§ 6851(a)(2)(B) annualization
    /// requirement).
    pub taxable_period_annualized: bool,
    /// Days from full-year return due date (or filing date,
    /// whichever later) to SNOD mailing (§ 6851(b) 60-day
    /// requirement).
    pub days_to_snod_mailing: u32,
    /// Days from termination assessment notice to payment
    /// or bond filing (§ 6863 10-day window).
    pub days_to_payment_or_bond: u32,
    /// Whether § 6863 bond was filed to stay collection.
    pub section_6863_bond_filed: bool,
    /// Whether Chief Counsel for IRS provided personal
    /// written approval (§ 7429(a)(1)(A) requirement).
    pub chief_counsel_personal_approval: bool,
    /// Whether taxpayer filed § 6213(a) Tax Court petition
    /// after SNOD issued.
    pub tax_court_petition_filed: bool,
    /// Whether termination assessment was abated when court
    /// determined unreasonable under § 6863(g).
    pub court_determined_unreasonable: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6851Result {
    pub termination_authorized: bool,
    pub annualization_compliant: bool,
    pub snod_within_60_day_window: bool,
    pub ten_day_payment_or_bond_compliant: bool,
    pub chief_counsel_approval_compliant: bool,
    pub stay_engaged_via_6863_bond: bool,
    pub abatement_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6851Input) -> Section6851Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let trigger_exists = !matches!(input.termination_trigger, TerminationTrigger::None);

    if !trigger_exists {
        failure_reasons.push(
            "26 USC § 6851(a)(1) — Secretary must find taxpayer designing to (A) depart from US or remove property, (B) conceal self or property, or (C) do any other act tending to prejudice or render ineffectual collection".to_string(),
        );
    }

    if !input.taxable_period_annualized {
        failure_reasons.push(
            "26 USC § 6851(a)(2)(B) — tax for terminated period must be computed by placing entire tax base on annual basis (annualize income for partial year)".to_string(),
        );
    }

    if !input.chief_counsel_personal_approval {
        failure_reasons.push(
            "26 USC § 7429(a)(1)(A) — Chief Counsel for IRS personal written approval required before § 6851 termination assessment or § 6331 levy".to_string(),
        );
    }

    let snod_compliant = input.days_to_snod_mailing <= 60;
    if !snod_compliant {
        failure_reasons.push(
            "26 USC § 6851(b) — SNOD must be issued within 60 days after later of (1) due date of return for full taxable year or (2) date taxpayer files such return".to_string(),
        );
    }

    let ten_day_compliant = input.days_to_payment_or_bond <= 10
        || input.section_6863_bond_filed;
    if !ten_day_compliant {
        failure_reasons.push(
            "26 USC § 6863 + IRM 5.17.15 — amount of termination assessment must be paid within 10 days unless § 6863 bond is filed to stay collection".to_string(),
        );
    }

    let authorized = trigger_exists
        && input.taxable_period_annualized
        && input.chief_counsel_personal_approval
        && snod_compliant
        && !input.court_determined_unreasonable;

    let notes: Vec<String> = vec![
        "26 USC § 6851(a)(1) — Secretary may TERMINATE taxpayer's taxable year immediately if finds taxpayer designing to (A) depart from US or remove property, (B) conceal self or property, or (C) do any other act tending to prejudice or render ineffectual collection".to_string(),
        "26 USC § 6851(a)(1)(C) — 'any other act' includes corporate liquidation distributing assets, dissipating assets, making oneself insolvent".to_string(),
        "26 USC § 6851(a)(2) — tax for terminated period computed (1) as if such period were taxable year of taxpayer AND (2) by placing entire tax base on annual basis (annualization)".to_string(),
        "26 USC § 6851(b) — SNOD must be issued within 60 days after LATER of (1) due date of return for full taxable year (including extensions) or (2) date taxpayer files such return".to_string(),
        "26 USC § 6851(c) — any amount collected as result of termination assessment treated as if collected on date assessment for entire taxable year were made".to_string(),
        "26 USC § 6851(d) cross-references: § 7429 review (30-day administrative + 90-day judicial); § 6863 bond stay; § 6213(a) Tax Court petition preserved by § 6851 60-day SNOD".to_string(),
        "§ 6851 vs § 6861 distinction: § 6851 terminates CURRENT/immediately preceding taxable year (income tax only) BEFORE return due date; § 6861 jeopardy assesses EXISTING DEFICIENCY (income/estate/gift) AFTER return filing or SNOD".to_string(),
        "§ 6851 + § 6863 interaction: amount of termination assessment must be paid within 10 days unless § 6863 bond filed to stay collection".to_string(),
        "IRM 4.15.1 + IRM 5.1.4 + IRM 5.17.15 — internal IRS procedural guidance on termination/jeopardy assessment determination and execution".to_string(),
    ];

    Section6851Result {
        termination_authorized: authorized,
        annualization_compliant: input.taxable_period_annualized,
        snod_within_60_day_window: snod_compliant,
        ten_day_payment_or_bond_compliant: ten_day_compliant,
        chief_counsel_approval_compliant: input.chief_counsel_personal_approval,
        stay_engaged_via_6863_bond: input.section_6863_bond_filed,
        abatement_engaged: input.court_determined_unreasonable,
        failure_reasons,
        citation: "26 USC § 6851(a)-(d); 26 CFR § 1.6851-1; § 6852; § 6861; § 6863; § 6213(a); § 6321; § 6331; § 7429; IRM 4.15.1; IRM 5.1.4; IRM 5.17.15",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6851Input {
        Section6851Input {
            termination_trigger: TerminationTrigger::DepartingOrRemovingProperty,
            taxable_period_annualized: true,
            days_to_snod_mailing: 30,
            days_to_payment_or_bond: 5,
            section_6863_bond_filed: false,
            chief_counsel_personal_approval: true,
            tax_court_petition_filed: false,
            court_determined_unreasonable: false,
        }
    }

    #[test]
    fn fully_compliant_termination_authorized() {
        let r = check(&valid_base());
        assert!(r.termination_authorized);
        assert!(r.annualization_compliant);
        assert!(r.snod_within_60_day_window);
        assert!(r.ten_day_payment_or_bond_compliant);
        assert!(r.chief_counsel_approval_compliant);
    }

    #[test]
    fn no_trigger_authorization_fails() {
        let mut i = valid_base();
        i.termination_trigger = TerminationTrigger::None;
        let r = check(&i);
        assert!(!r.termination_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6851(a)(1)") && f.contains("depart from US")));
    }

    #[test]
    fn departing_or_removing_property_trigger_authorized() {
        let mut i = valid_base();
        i.termination_trigger = TerminationTrigger::DepartingOrRemovingProperty;
        let r = check(&i);
        assert!(r.termination_authorized);
    }

    #[test]
    fn concealing_self_or_property_trigger_authorized() {
        let mut i = valid_base();
        i.termination_trigger = TerminationTrigger::ConcealingSelfOrProperty;
        let r = check(&i);
        assert!(r.termination_authorized);
    }

    #[test]
    fn other_jeopardizing_act_or_liquidation_trigger_authorized() {
        let mut i = valid_base();
        i.termination_trigger = TerminationTrigger::OtherJeopardizingActOrLiquidation;
        let r = check(&i);
        assert!(r.termination_authorized);
    }

    #[test]
    fn no_annualization_fails() {
        let mut i = valid_base();
        i.taxable_period_annualized = false;
        let r = check(&i);
        assert!(!r.termination_authorized);
        assert!(!r.annualization_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6851(a)(2)(B)") && f.contains("annualize")));
    }

    #[test]
    fn no_chief_counsel_approval_fails() {
        let mut i = valid_base();
        i.chief_counsel_personal_approval = false;
        let r = check(&i);
        assert!(!r.termination_authorized);
        assert!(!r.chief_counsel_approval_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(a)(1)(A)") && f.contains("Chief Counsel")));
    }

    #[test]
    fn snod_60_day_boundary_compliant() {
        let mut i = valid_base();
        i.days_to_snod_mailing = 60;
        let r = check(&i);
        assert!(r.snod_within_60_day_window);
        assert!(r.termination_authorized);
    }

    #[test]
    fn snod_61_day_violation() {
        let mut i = valid_base();
        i.days_to_snod_mailing = 61;
        let r = check(&i);
        assert!(!r.snod_within_60_day_window);
        assert!(!r.termination_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6851(b)") && f.contains("60 days")));
    }

    #[test]
    fn ten_day_payment_boundary_compliant() {
        let mut i = valid_base();
        i.days_to_payment_or_bond = 10;
        let r = check(&i);
        assert!(r.ten_day_payment_or_bond_compliant);
    }

    #[test]
    fn eleven_day_payment_violation_without_bond() {
        let mut i = valid_base();
        i.days_to_payment_or_bond = 11;
        i.section_6863_bond_filed = false;
        let r = check(&i);
        assert!(!r.ten_day_payment_or_bond_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6863") && f.contains("10 days")));
    }

    #[test]
    fn eleven_day_payment_with_6863_bond_compliant() {
        let mut i = valid_base();
        i.days_to_payment_or_bond = 100;
        i.section_6863_bond_filed = true;
        let r = check(&i);
        assert!(r.ten_day_payment_or_bond_compliant);
        assert!(r.stay_engaged_via_6863_bond);
    }

    #[test]
    fn court_determined_unreasonable_engages_abatement() {
        let mut i = valid_base();
        i.court_determined_unreasonable = true;
        let r = check(&i);
        assert!(r.abatement_engaged);
        assert!(!r.termination_authorized);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6851(a)-(d)"));
        assert!(r.citation.contains("26 CFR § 1.6851-1"));
        assert!(r.citation.contains("§ 6852"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6863"));
        assert!(r.citation.contains("§ 6213(a)"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 7429"));
        assert!(r.citation.contains("IRM 4.15.1"));
        assert!(r.citation.contains("IRM 5.1.4"));
        assert!(r.citation.contains("IRM 5.17.15"));
    }

    #[test]
    fn note_pins_section_a1_termination_authority() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851(a)(1)")
            && n.contains("TERMINATE")
            && n.contains("depart from US")
            && n.contains("conceal")
            && n.contains("any other act")));
    }

    #[test]
    fn note_pins_section_a1C_corporate_liquidation_scope() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851(a)(1)(C)")
            && n.contains("corporate liquidation")
            && n.contains("dissipating")));
    }

    #[test]
    fn note_pins_section_a2_annualization() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851(a)(2)")
            && n.contains("annual basis")
            && n.contains("annualization")));
    }

    #[test]
    fn note_pins_section_b_60_day_snod() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851(b)")
            && n.contains("60 days")
            && n.contains("LATER")));
    }

    #[test]
    fn note_pins_section_c_treatment_of_amounts_collected() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851(c)")
            && n.contains("treated as if collected")));
    }

    #[test]
    fn note_pins_6851_vs_6861_distinction() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851 vs § 6861")
            && n.contains("BEFORE return due date")
            && n.contains("EXISTING DEFICIENCY")));
    }

    #[test]
    fn note_pins_6851_6863_10_day_interaction() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6851 + § 6863")
            && n.contains("10 days")));
    }

    #[test]
    fn note_pins_irm_procedural_guidance() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 4.15.1")
            && n.contains("IRM 5.1.4")
            && n.contains("IRM 5.17.15")));
    }

    #[test]
    fn termination_trigger_truth_table_four_cells() {
        for (trigger, exp_authorized) in [
            (TerminationTrigger::DepartingOrRemovingProperty, true),
            (TerminationTrigger::ConcealingSelfOrProperty, true),
            (TerminationTrigger::OtherJeopardizingActOrLiquidation, true),
            (TerminationTrigger::None, false),
        ] {
            let mut i = valid_base();
            i.termination_trigger = trigger;
            let r = check(&i);
            assert_eq!(r.termination_authorized, exp_authorized);
        }
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.termination_trigger = TerminationTrigger::None;
        i.taxable_period_annualized = false;
        i.chief_counsel_personal_approval = false;
        i.days_to_snod_mailing = 100;
        i.days_to_payment_or_bond = 30;
        i.section_6863_bond_filed = false;
        let r = check(&i);
        assert!(!r.termination_authorized);
        assert_eq!(r.failure_reasons.len(), 5);
    }

    #[test]
    fn court_unreasonable_blocks_authorization_invariant() {
        let mut i = valid_base();
        i.court_determined_unreasonable = true;
        let r = check(&i);
        assert!(r.abatement_engaged);
        assert!(!r.termination_authorized);
    }

    #[test]
    fn section_6863_bond_filed_overrides_10_day_requirement_invariant() {
        let mut i_no_bond = valid_base();
        i_no_bond.days_to_payment_or_bond = 365;
        i_no_bond.section_6863_bond_filed = false;
        let r_no_bond = check(&i_no_bond);
        assert!(!r_no_bond.ten_day_payment_or_bond_compliant);

        let mut i_bond = valid_base();
        i_bond.days_to_payment_or_bond = 365;
        i_bond.section_6863_bond_filed = true;
        let r_bond = check(&i_bond);
        assert!(r_bond.ten_day_payment_or_bond_compliant);
        assert!(r_bond.stay_engaged_via_6863_bond);
    }

    #[test]
    fn snod_60_day_boundary_invariant() {
        let mut i_at = valid_base();
        i_at.days_to_snod_mailing = 60;
        let r_at = check(&i_at);
        assert!(r_at.snod_within_60_day_window);

        let mut i_over = valid_base();
        i_over.days_to_snod_mailing = 61;
        let r_over = check(&i_over);
        assert!(!r_over.snod_within_60_day_window);
    }
}
