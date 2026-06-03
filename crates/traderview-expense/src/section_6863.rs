//! IRC § 6863 — Stay of collection of jeopardy assessments.
//! The procedural pressure-relief valve when § 6861 (income/
//! estate/gift jeopardy assessment), § 6862 (other-tax
//! jeopardy assessment), § 6851 (termination assessment of
//! income tax), or § 6852 (termination assessment in cases
//! of qualified person) has been imposed. Trader-procedural-
//! critical because § 6863 permits the trader-taxpayer to
//! STAY immediate § 6321 lien attachment and § 6331 levy
//! seizure by filing a bond in amount of stayed assessment
//! plus interest. Companion to § 6861 (income/estate/gift
//! jeopardy assessment), § 6862 (other-tax jeopardy
//! assessment), § 6851 (termination assessment), § 6852
//! (termination of taxable year of qualified person),
//! § 6213(a) (Tax Court petition window), § 6321 (lien),
//! § 6331 (levy), § 7429 (review of jeopardy procedures).
//!
//! **§ 6863(a) Bond to stay collection** — when an
//! assessment has been made under § 6851, § 6852, § 6861,
//! or § 6862, the collection of the whole or any amount of
//! such assessment may be stayed by **filing with the
//! Secretary, within such time as may be fixed by
//! regulations, a bond in an amount equal to the amount as
//! to which the stay is desired (not exceeding amount of
//! jeopardy assessment plus interest)**, conditioned upon
//! the payment of the amount (together with interest
//! thereon) at the time at which, but for the making of
//! such assessment, such amount would be due. Upon filing
//! the bond, the collection of so much of the amount
//! assessed as is covered by the bond shall be STAYED.
//!
//! **§ 6863(b)(1) Bond filed before § 6213(a) petition** —
//! if the bond is given **before the taxpayer has filed
//! § 6213(a) Tax Court petition**, the bond shall contain a
//! further condition that if a petition is not filed within
//! § 6213(a) 90-day window (or 150 days outside US), then
//! the amount, the collection of which is stayed by the
//! bond, will be paid on notice and demand at any time
//! after the expiration of such period, together with
//! interest thereon from the date of the jeopardy notice
//! and demand to the date of notice and demand under this
//! paragraph.
//!
//! **§ 6863(b)(2) Bond conditioned on Tax Court decision**
//! — the bond shall be conditioned upon the payment of so
//! much of such assessment (collection of which is stayed
//! by the bond) as is **not abated by a decision of the
//! Tax Court which has become final**. If the Tax Court
//! determines that the amount assessed is greater than the
//! amount which should have been assessed, then when the
//! decision of the Tax Court is rendered the bond shall, at
//! the request of the taxpayer, be **proportionately
//! reduced**.
//!
//! **§ 6863(b)(3)(A) Stay of sale of seized property
//! pending Tax Court decision** — except as provided in
//! § 6863(b)(3)(B), where a jeopardy assessment has been
//! made under § 6862(a), the property seized for the
//! collection of the tax **shall not be sold** if a civil
//! action under § 7429(b) has been commenced on or before
//! the day on which the district court judgment in such
//! action becomes final.
//!
//! **§ 6863(b)(3)(B) Exceptions** — sale prohibition does
//! not apply when: (i) taxpayer consents to sale; (ii)
//! Secretary determines expenses of conservation and
//! maintenance would greatly reduce net proceeds; OR (iii)
//! property is perishable.
//!
//! **§ 6863(g) Abatement if jeopardy does not exist** — if
//! the appropriate court determines that the making of the
//! assessment was unreasonable or that the amount assessed
//! or demanded was inappropriate, then such court may order
//! the Secretary to ABATE such assessment, redetermine the
//! amount of such assessment, or take such other action as
//! the court finds appropriate.
//!
//! Citations: 26 USC § 6863(a)-(g); 26 CFR § 301.6863-1;
//! § 6851 (termination assessment of income tax); § 6852
//! (termination — qualified person); § 6861 (jeopardy
//! assessment income/estate/gift); § 6862 (jeopardy
//! assessment other taxes); § 6213(a) (Tax Court petition
//! window); § 6321 (lien); § 6331 (levy); § 7429 (review of
//! jeopardy procedures); IRM 5.17.15 (Termination and
//! Jeopardy Assessments and Jeopardy Collection); IRM
//! 5.1.4 (Jeopardy, Termination, Quick and Prompt
//! Assessments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnderlyingAssessment {
    /// § 6851 termination assessment of income tax.
    Section6851Termination,
    /// § 6852 termination assessment in cases of qualified
    /// person.
    Section6852QualifiedPersonTermination,
    /// § 6861 jeopardy assessment of income, estate, gift,
    /// and certain excise taxes.
    Section6861Jeopardy,
    /// § 6862 jeopardy assessment of other taxes.
    Section6862OtherTaxesJeopardy,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SaleException {
    /// No exception applies — sale prohibited pending § 7429
    /// district court judgment.
    None,
    /// § 6863(b)(3)(B)(i) taxpayer consents to sale.
    TaxpayerConsentsToSale,
    /// § 6863(b)(3)(B)(ii) Secretary determines expenses of
    /// conservation and maintenance would greatly reduce net
    /// proceeds.
    ExcessiveConservationCosts,
    /// § 6863(b)(3)(B)(iii) property is perishable.
    PropertyPerishable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6863Input {
    pub underlying_assessment: UnderlyingAssessment,
    /// Amount of jeopardy assessment + interest in cents.
    pub jeopardy_assessment_plus_interest_cents: u64,
    /// Amount of bond filed by taxpayer in cents.
    pub bond_amount_cents: u64,
    /// Whether bond was filed within Secretary-fixed time
    /// (regulations 26 CFR § 301.6863-1).
    pub bond_filed_within_time: bool,
    /// Whether bond was filed BEFORE § 6213(a) Tax Court
    /// petition (triggers § 6863(b)(1) additional condition).
    pub bond_filed_before_tax_court_petition: bool,
    /// Whether § 6213(a) petition window has expired without
    /// petition filing.
    pub petition_window_expired_without_petition: bool,
    /// Whether Tax Court has rendered final decision.
    pub tax_court_final_decision: bool,
    /// Whether Tax Court determined deficiency LESS than
    /// jeopardy assessment (triggers § 6863(b)(2)
    /// proportional reduction at taxpayer request).
    pub tax_court_determined_less_than_jeopardy: bool,
    /// Whether taxpayer requested proportional reduction of
    /// bond.
    pub taxpayer_requested_proportional_reduction: bool,
    /// Whether § 6862(a) underlying assessment involves
    /// seized property pending § 7429(b) civil action.
    pub section_6862_seized_property_pending_7429: bool,
    /// § 6863(b)(3)(B) sale exception (if any).
    pub sale_exception: SaleException,
    /// Whether appropriate court determined jeopardy
    /// assessment unreasonable OR amount inappropriate
    /// (§ 6863(g) abatement trigger).
    pub court_determined_unreasonable_or_inappropriate: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6863Result {
    pub stay_engaged: bool,
    pub bond_within_jeopardy_amount_cap: bool,
    pub stay_amount_cents: u64,
    pub pre_petition_condition_engaged: bool,
    pub tax_court_reduction_available: bool,
    pub sale_of_seized_property_prohibited: bool,
    pub abatement_required: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6863Input) -> Section6863Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.bond_filed_within_time {
        failure_reasons.push(
            "26 USC § 6863(a) + 26 CFR § 301.6863-1 — bond must be filed within time fixed by Secretary regulations to engage stay".to_string(),
        );
    }

    let cap = input.jeopardy_assessment_plus_interest_cents;
    let within_cap = input.bond_amount_cents <= cap;

    if !within_cap {
        failure_reasons.push(
            "26 USC § 6863(a) — bond amount may not exceed amount of jeopardy assessment plus interest".to_string(),
        );
    }

    let pre_petition_condition = input.bond_filed_before_tax_court_petition;

    if pre_petition_condition && input.petition_window_expired_without_petition {
        failure_reasons.push(
            "26 USC § 6863(b)(1) — if § 6213(a) petition not filed within 90-day window (150 days outside US), stayed amount becomes payable on notice and demand together with interest from date of jeopardy notice".to_string(),
        );
    }

    let reduction_available = input.tax_court_final_decision
        && input.tax_court_determined_less_than_jeopardy
        && input.taxpayer_requested_proportional_reduction;

    let sale_prohibition_engaged = matches!(
        input.underlying_assessment,
        UnderlyingAssessment::Section6862OtherTaxesJeopardy
    ) && input.section_6862_seized_property_pending_7429
        && matches!(input.sale_exception, SaleException::None);

    let stay_engaged = input.bond_filed_within_time && within_cap;
    let stay_amount = if stay_engaged {
        input.bond_amount_cents.min(cap)
    } else {
        0
    };

    let abatement = input.court_determined_unreasonable_or_inappropriate;

    let notes: Vec<String> = vec![
        "26 USC § 6863(a) — collection of jeopardy assessment under § 6851, § 6852, § 6861, or § 6862 may be stayed by filing with Secretary a bond in amount equal to amount of stay desired (not exceeding amount of jeopardy assessment plus interest)".to_string(),
        "26 USC § 6863(a) — upon filing of bond, the collection of so much of the amount assessed as is covered by the bond shall be STAYED; bond conditioned on payment when assessment would otherwise be due".to_string(),
        "26 USC § 6863(b)(1) — if bond filed BEFORE § 6213(a) Tax Court petition, bond shall contain further condition: if petition not filed within 90-day window (150 days outside US), stayed amount becomes payable on notice and demand at any time after expiration of such period, together with interest from jeopardy notice and demand date".to_string(),
        "26 USC § 6863(b)(2) — bond conditioned on payment of so much of assessment as is NOT abated by final Tax Court decision; if Tax Court determines amount assessed greater than amount which should have been assessed, bond at taxpayer's request shall be PROPORTIONATELY REDUCED".to_string(),
        "26 USC § 6863(b)(3)(A) — for § 6862(a) jeopardy assessment, property seized SHALL NOT BE SOLD if § 7429(b) civil action commenced on or before day district court judgment becomes final".to_string(),
        "26 USC § 6863(b)(3)(B) — sale-prohibition exceptions: (i) taxpayer consents to sale; (ii) Secretary determines expenses of conservation/maintenance would greatly reduce net proceeds; (iii) property is perishable".to_string(),
        "26 USC § 6863(g) — if appropriate court determines making of assessment was unreasonable OR amount assessed/demanded was inappropriate, court may order Secretary to ABATE assessment, redetermine amount, or take other action court finds appropriate".to_string(),
        "Cross-references: § 6863 is procedural relief for § 6861 (income/estate/gift jeopardy) + § 6862 (other-tax jeopardy) + § 6851 (income tax termination) + § 6852 (qualified-person termination); preserves § 6213(a) Tax Court petition right; subject to § 7429 30-day administrative + 90-day judicial review procedural framework".to_string(),
        "IRM 5.17.15 (Termination and Jeopardy Assessments and Jeopardy Collection) + IRM 5.1.4 — internal IRS procedural guidance on bond acceptance and stay processing".to_string(),
    ];

    Section6863Result {
        stay_engaged,
        bond_within_jeopardy_amount_cap: within_cap,
        stay_amount_cents: stay_amount,
        pre_petition_condition_engaged: pre_petition_condition,
        tax_court_reduction_available: reduction_available,
        sale_of_seized_property_prohibited: sale_prohibition_engaged,
        abatement_required: abatement,
        failure_reasons,
        citation: "26 USC § 6863(a)-(g); 26 CFR § 301.6863-1; § 6851; § 6852; § 6861; § 6862; § 6213(a); § 6321; § 6331; § 7429; IRM 5.17.15; IRM 5.1.4",
        notes,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn valid_base() -> Section6863Input {
        Section6863Input {
            underlying_assessment: UnderlyingAssessment::Section6861Jeopardy,
            jeopardy_assessment_plus_interest_cents: 100_000_000,
            bond_amount_cents: 100_000_000,
            bond_filed_within_time: true,
            bond_filed_before_tax_court_petition: false,
            petition_window_expired_without_petition: false,
            tax_court_final_decision: false,
            tax_court_determined_less_than_jeopardy: false,
            taxpayer_requested_proportional_reduction: false,
            section_6862_seized_property_pending_7429: false,
            sale_exception: SaleException::None,
            court_determined_unreasonable_or_inappropriate: false,
        }
    }

    #[test]
    fn bond_at_cap_compliant_stay_engaged() {
        let r = check(&valid_base());
        assert!(r.stay_engaged);
        assert_eq!(r.stay_amount_cents, 100_000_000);
        assert!(r.bond_within_jeopardy_amount_cap);
    }

    #[test]
    fn bond_under_cap_compliant_stay_engaged_partial() {
        let mut i = valid_base();
        i.bond_amount_cents = 50_000_000;
        let r = check(&i);
        assert!(r.stay_engaged);
        assert_eq!(r.stay_amount_cents, 50_000_000);
    }

    #[test]
    fn bond_above_cap_violation() {
        let mut i = valid_base();
        i.bond_amount_cents = 100_000_001;
        let r = check(&i);
        assert!(!r.bond_within_jeopardy_amount_cap);
        assert!(!r.stay_engaged);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6863(a)") && f.contains("plus interest")));
    }

    #[test]
    fn bond_not_filed_within_time_violation() {
        let mut i = valid_base();
        i.bond_filed_within_time = false;
        let r = check(&i);
        assert!(!r.stay_engaged);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6863(a)") && f.contains("§ 301.6863-1")));
    }

    #[test]
    fn pre_petition_condition_engaged_when_filed_before_petition() {
        let mut i = valid_base();
        i.bond_filed_before_tax_court_petition = true;
        let r = check(&i);
        assert!(r.pre_petition_condition_engaged);
    }

    #[test]
    fn pre_petition_condition_petition_missed_payable_on_demand() {
        let mut i = valid_base();
        i.bond_filed_before_tax_court_petition = true;
        i.petition_window_expired_without_petition = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6863(b)(1)") && f.contains("90-day")));
    }

    #[test]
    fn tax_court_reduction_available_with_full_chain() {
        let mut i = valid_base();
        i.tax_court_final_decision = true;
        i.tax_court_determined_less_than_jeopardy = true;
        i.taxpayer_requested_proportional_reduction = true;
        let r = check(&i);
        assert!(r.tax_court_reduction_available);
    }

    #[test]
    fn tax_court_reduction_unavailable_without_taxpayer_request() {
        let mut i = valid_base();
        i.tax_court_final_decision = true;
        i.tax_court_determined_less_than_jeopardy = true;
        i.taxpayer_requested_proportional_reduction = false;
        let r = check(&i);
        assert!(!r.tax_court_reduction_available);
    }

    #[test]
    fn tax_court_reduction_unavailable_without_less_determination() {
        let mut i = valid_base();
        i.tax_court_final_decision = true;
        i.tax_court_determined_less_than_jeopardy = false;
        i.taxpayer_requested_proportional_reduction = true;
        let r = check(&i);
        assert!(!r.tax_court_reduction_available);
    }

    #[test]
    fn sale_prohibition_engaged_for_6862_pending_7429() {
        let mut i = valid_base();
        i.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
        i.section_6862_seized_property_pending_7429 = true;
        i.sale_exception = SaleException::None;
        let r = check(&i);
        assert!(r.sale_of_seized_property_prohibited);
    }

    #[test]
    fn sale_exception_taxpayer_consent_disengages_prohibition() {
        let mut i = valid_base();
        i.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
        i.section_6862_seized_property_pending_7429 = true;
        i.sale_exception = SaleException::TaxpayerConsentsToSale;
        let r = check(&i);
        assert!(!r.sale_of_seized_property_prohibited);
    }

    #[test]
    fn sale_exception_excessive_conservation_costs_disengages_prohibition() {
        let mut i = valid_base();
        i.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
        i.section_6862_seized_property_pending_7429 = true;
        i.sale_exception = SaleException::ExcessiveConservationCosts;
        let r = check(&i);
        assert!(!r.sale_of_seized_property_prohibited);
    }

    #[test]
    fn sale_exception_perishable_property_disengages_prohibition() {
        let mut i = valid_base();
        i.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
        i.section_6862_seized_property_pending_7429 = true;
        i.sale_exception = SaleException::PropertyPerishable;
        let r = check(&i);
        assert!(!r.sale_of_seized_property_prohibited);
    }

    #[test]
    fn sale_prohibition_not_engaged_for_non_6862_assessment() {
        let mut i = valid_base();
        i.underlying_assessment = UnderlyingAssessment::Section6861Jeopardy;
        i.section_6862_seized_property_pending_7429 = true;
        let r = check(&i);
        assert!(!r.sale_of_seized_property_prohibited);
    }

    #[test]
    fn court_unreasonable_or_inappropriate_determination_engages_abatement() {
        let mut i = valid_base();
        i.court_determined_unreasonable_or_inappropriate = true;
        let r = check(&i);
        assert!(r.abatement_required);
    }

    #[test]
    fn underlying_assessment_truth_table_four_cells() {
        for underlying in [
            UnderlyingAssessment::Section6851Termination,
            UnderlyingAssessment::Section6852QualifiedPersonTermination,
            UnderlyingAssessment::Section6861Jeopardy,
            UnderlyingAssessment::Section6862OtherTaxesJeopardy,
        ] {
            let mut i = valid_base();
            i.underlying_assessment = underlying;
            let r = check(&i);
            assert!(r.stay_engaged);
        }
    }

    #[test]
    fn sale_exception_truth_table_four_cells() {
        for (exception, exp_prohibited) in [
            (SaleException::None, true),
            (SaleException::TaxpayerConsentsToSale, false),
            (SaleException::ExcessiveConservationCosts, false),
            (SaleException::PropertyPerishable, false),
        ] {
            let mut i = valid_base();
            i.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
            i.section_6862_seized_property_pending_7429 = true;
            i.sale_exception = exception;
            let r = check(&i);
            assert_eq!(
                r.sale_of_seized_property_prohibited, exp_prohibited,
                "exception={:?} expected prohibited={}",
                exception, exp_prohibited
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6863(a)-(g)"));
        assert!(r.citation.contains("26 CFR § 301.6863-1"));
        assert!(r.citation.contains("§ 6851"));
        assert!(r.citation.contains("§ 6852"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6862"));
        assert!(r.citation.contains("§ 6213(a)"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 7429"));
        assert!(r.citation.contains("IRM 5.17.15"));
        assert!(r.citation.contains("IRM 5.1.4"));
    }

    #[test]
    fn note_pins_subsection_a_bond_to_stay_collection() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6863(a)") && n.contains("STAYED")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6863(a)") && n.contains("plus interest")));
    }

    #[test]
    fn note_pins_subsection_b1_pre_petition_condition() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6863(b)(1)")
            && n.contains("§ 6213(a)")
            && n.contains("90-day")
            && n.contains("150 days outside US")));
    }

    #[test]
    fn note_pins_subsection_b2_proportional_reduction() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6863(b)(2)")
            && n.contains("PROPORTIONATELY REDUCED")));
    }

    #[test]
    fn note_pins_subsection_b3A_sale_prohibition() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6863(b)(3)(A)")
            && n.contains("SHALL NOT BE SOLD")
            && n.contains("§ 7429(b)")));
    }

    #[test]
    fn note_pins_subsection_b3B_three_sale_exceptions() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6863(b)(3)(B)")
            && n.contains("taxpayer consents")
            && n.contains("conservation/maintenance")
            && n.contains("perishable")));
    }

    #[test]
    fn note_pins_subsection_g_abatement() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6863(g)")
            && n.contains("ABATE")));
    }

    #[test]
    fn note_pins_cross_references_four_underlying_assessments() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6861")
            && n.contains("§ 6862")
            && n.contains("§ 6851")
            && n.contains("§ 6852")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.bond_filed_within_time = false;
        i.bond_amount_cents = 999_999_999;
        i.bond_filed_before_tax_court_petition = true;
        i.petition_window_expired_without_petition = true;
        let r = check(&i);
        assert!(!r.stay_engaged);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn bond_amount_cap_invariant() {
        let mut i_at = valid_base();
        i_at.bond_amount_cents = 100_000_000;
        let r_at = check(&i_at);
        assert!(r_at.bond_within_jeopardy_amount_cap);

        let mut i_over = valid_base();
        i_over.bond_amount_cents = 100_000_001;
        let r_over = check(&i_over);
        assert!(!r_over.bond_within_jeopardy_amount_cap);
    }

    #[test]
    fn sale_prohibition_only_for_6862_invariant() {
        for underlying in [
            UnderlyingAssessment::Section6851Termination,
            UnderlyingAssessment::Section6852QualifiedPersonTermination,
            UnderlyingAssessment::Section6861Jeopardy,
        ] {
            let mut i = valid_base();
            i.underlying_assessment = underlying;
            i.section_6862_seized_property_pending_7429 = true;
            let r = check(&i);
            assert!(!r.sale_of_seized_property_prohibited);
        }

        let mut i_6862 = valid_base();
        i_6862.underlying_assessment = UnderlyingAssessment::Section6862OtherTaxesJeopardy;
        i_6862.section_6862_seized_property_pending_7429 = true;
        let r_6862 = check(&i_6862);
        assert!(r_6862.sale_of_seized_property_prohibited);
    }

    #[test]
    fn defensive_zero_bond_no_stay() {
        let mut i = valid_base();
        i.bond_amount_cents = 0;
        let r = check(&i);
        assert!(r.stay_engaged);
        assert_eq!(r.stay_amount_cents, 0);
    }

    #[test]
    fn defensive_u64_max_jeopardy_amount() {
        let mut i = valid_base();
        i.jeopardy_assessment_plus_interest_cents = u64::MAX;
        i.bond_amount_cents = u64::MAX;
        let r = check(&i);
        assert!(r.bond_within_jeopardy_amount_cap);
        assert_eq!(r.stay_amount_cents, u64::MAX);
    }
}
