//! IRC § 6695A — Substantial and gross valuation
//! misstatements attributable to incorrect appraisals.
//! Added by Pension Protection Act of 2006 § 1219 to
//! penalize appraisers whose appraisals support substantial
//! or gross valuation misstatements. Trader-relevant for
//! traders engaging in: art donations (§ 170 charitable
//! contribution), conservation easements (§ 170(h)), facade
//! easements, real estate exchanges (§ 1031), partnership
//! interest valuations, syndicated conservation easement
//! deductions (§ 6707A listed transaction crossover), and
//! estate/gift tax valuations (§§ 2031, 2512). Companion to
//! § 6662(e) substantial valuation misstatement; § 6662(g)
//! substantial estate/gift valuation understatement;
//! § 6662(h) gross valuation misstatement; § 170(f)(11)
//! qualified appraisal requirements; § 6707A reportable
//! transaction penalty; Circular 230 appraiser conduct.
//!
//! **§ 6695A(a) Imposition of penalty** — if (1) a person
//! prepares an appraisal of the value of property AND (2)
//! such person KNEW OR REASONABLY SHOULD HAVE KNOWN that
//! the appraisal would be used in connection with a return
//! or claim for refund AND (3) the claimed value of the
//! property on a return or claim for refund which is based
//! on such appraisal results in:
//! 1. A **substantial valuation misstatement** within
//!    meaning of § 6662(e) — claimed value **150% or more**
//!    of correct amount;
//! 2. A **substantial estate or gift tax valuation
//!    understatement** within meaning of § 6662(g) — value
//!    reported on estate/gift return is **65% or less** of
//!    correct amount;
//! 3. A **gross valuation misstatement** within meaning of
//!    § 6662(h) — claimed value **200% or more** of correct
//!    amount;
//!
//! THEN such person shall pay a penalty in the amount
//! determined under subsection (b).
//!
//! **§ 6695A(b) Amount of penalty** — penalty equals the
//! LESSER OF (1) the GREATER of (a) 10% of the
//! underpayment attributable to the misstatement OR (b)
//! $1,000 AND (2) 125% of the gross income received by
//! such person from the preparation of the appraisal.
//!
//! **§ 6695A(c) Exception for good-faith determination** —
//! no penalty shall be imposed under § 6695A(a) if the
//! person establishes to the satisfaction of the Secretary
//! that the value established in the appraisal was **MORE
//! LIKELY THAN NOT** the proper value (51% confidence
//! standard).
//!
//! **Effective dates**:
//! - General rule — applies to appraisals prepared with
//!   respect to returns or submissions filed **after August
//!   17, 2006** (Pension Protection Act § 1219 enactment
//!   date).
//! - Facade easement special rule — if appraisal relates to
//!   a **facade easement donation**, the penalty applies to
//!   returns filed **after July 25, 2006**.
//!
//! Citations: 26 USC § 6695A(a)-(c); Pension Protection
//! Act of 2006 § 1219; 26 CFR § 1.6695A; IRM 20.1.12
//! (Penalties Applicable to Incorrect Appraisals); §
//! 6662(e) substantial valuation misstatement; § 6662(g)
//! estate/gift valuation understatement; § 6662(h) gross
//! valuation misstatement; § 170(f)(11) qualified appraisal;
//! § 6707A listed transaction; Circular 230 § 10.50
//! appraiser conduct.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MisstatementCategory {
    /// § 6662(e) substantial valuation misstatement —
    /// claimed value ≥ 150% of correct amount.
    SubstantialValuationMisstatement,
    /// § 6662(g) substantial estate/gift tax valuation
    /// understatement — reported value ≤ 65% of correct
    /// amount.
    EstateGiftValuationUnderstatement,
    /// § 6662(h) gross valuation misstatement — claimed
    /// value ≥ 200% of correct amount.
    GrossValuationMisstatement,
    /// No qualifying misstatement.
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppraisalType {
    /// General property appraisal (subject to August 17,
    /// 2006 effective date).
    GeneralProperty,
    /// Facade easement donation appraisal (subject to July
    /// 25, 2006 effective date special rule).
    FacadeEasementDonation,
    /// Conservation easement appraisal under § 170(h).
    ConservationEasement,
    /// Art donation appraisal under § 170(a).
    ArtDonation,
    /// Estate or gift tax appraisal under §§ 2031, 2512.
    EstateOrGiftTax,
    /// Syndicated conservation easement (§ 6707A listed
    /// transaction crossover).
    SyndicatedConservationEasement,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6695AInput {
    pub misstatement_category: MisstatementCategory,
    pub appraisal_type: AppraisalType,
    /// Whether appraiser knew or reasonably should have
    /// known the appraisal would be used on a return.
    pub knew_or_should_have_known_appraisal_use: bool,
    /// Underpayment attributable to misstatement in cents.
    pub underpayment_cents: u64,
    /// Gross income received by appraiser from preparing
    /// the appraisal in cents.
    pub appraiser_gross_income_cents: u64,
    /// Whether appraiser establishes more-likely-than-not
    /// (51% confidence) that value was proper.
    pub more_likely_than_not_proper_value: bool,
    /// Whether return filed after August 17, 2006 (general
    /// effective date).
    pub return_filed_after_august_17_2006: bool,
    /// Whether return filed after July 25, 2006 (facade
    /// easement special effective date).
    pub return_filed_after_july_25_2006: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6695AResult {
    pub penalty_imposed: bool,
    pub good_faith_exception_engaged: bool,
    pub effective_date_satisfied: bool,
    pub penalty_amount_cents: u64,
    pub greater_of_amount_cents: u64,
    pub income_cap_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6695AInput) -> Section6695AResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let misstatement_present =
        !matches!(input.misstatement_category, MisstatementCategory::None);

    let effective_date_satisfied = if matches!(
        input.appraisal_type,
        AppraisalType::FacadeEasementDonation
    ) {
        input.return_filed_after_july_25_2006
    } else {
        input.return_filed_after_august_17_2006
    };

    if !effective_date_satisfied {
        failure_reasons.push(
            "Pension Protection Act of 2006 § 1219 — § 6695A penalty applies only to appraisals on returns filed after August 17, 2006 (or July 25, 2006 for facade easement donations)".to_string(),
        );
    }

    let knew_or_should = input.knew_or_should_have_known_appraisal_use;

    let good_faith_exception = input.more_likely_than_not_proper_value;

    let imposed = misstatement_present
        && knew_or_should
        && effective_date_satisfied
        && !good_faith_exception;

    let ten_percent = input.underpayment_cents / 10;
    let greater_of = ten_percent.max(100_000);
    let income_cap = input.appraiser_gross_income_cents.saturating_mul(125) / 100;
    let penalty_amount = if imposed {
        greater_of.min(income_cap)
    } else {
        0
    };

    if misstatement_present && !knew_or_should {
        failure_reasons.push(
            "26 USC § 6695A(a)(2) — appraiser must have KNOWN OR REASONABLY SHOULD HAVE KNOWN that the appraisal would be used in connection with return or claim for refund".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 6695A(a) — penalty imposed when (1) person prepares appraisal AND (2) knew or reasonably should have known appraisal would be used on return or claim for refund AND (3) claimed value results in § 6662(e) substantial, § 6662(g) estate/gift, or § 6662(h) gross valuation misstatement".to_string(),
        "26 USC § 6695A(b) — penalty equals LESSER OF (1) greater of 10% of underpayment OR $1,000 AND (2) 125% of gross income received from appraisal preparation".to_string(),
        "26 USC § 6695A(c) — good-faith exception: no penalty if person establishes value was MORE LIKELY THAN NOT (51% confidence) the proper value to satisfaction of Secretary".to_string(),
        "26 USC § 6662(e) substantial valuation misstatement — claimed value ≥ 150% of correct amount".to_string(),
        "26 USC § 6662(g) substantial estate/gift tax valuation understatement — reported value ≤ 65% of correct amount".to_string(),
        "26 USC § 6662(h) gross valuation misstatement — claimed value ≥ 200% of correct amount; doubles § 6662 penalty rate from 20% to 40%".to_string(),
        "Effective dates — general rule applies to returns filed AFTER AUGUST 17, 2006 (Pension Protection Act § 1219 enactment); facade easement donation special rule applies to returns filed AFTER JULY 25, 2006".to_string(),
        "Pension Protection Act of 2006 § 1219 added § 6695A; companion to § 170(f)(11) qualified appraisal requirements + § 6707A reportable transaction penalty + § 6662 accuracy-related penalty + Circular 230 § 10.50 appraiser conduct".to_string(),
        "Trader-relevant applications: art donations + conservation easements (§ 170(h)) + facade easements + § 1031 real estate exchanges + partnership interest valuations + syndicated conservation easement deductions (§ 6707A listed transaction crossover) + estate/gift tax valuations".to_string(),
        "IRM 20.1.12 (Penalties Applicable to Incorrect Appraisals) — internal IRS procedural guidance on § 6695A assertion and abatement".to_string(),
    ];

    Section6695AResult {
        penalty_imposed: imposed,
        good_faith_exception_engaged: good_faith_exception,
        effective_date_satisfied,
        penalty_amount_cents: penalty_amount,
        greater_of_amount_cents: greater_of,
        income_cap_cents: income_cap,
        failure_reasons,
        citation: "26 USC § 6695A(a)-(c); Pension Protection Act of 2006 § 1219; 26 CFR § 1.6695A; IRM 20.1.12; § 6662(e); § 6662(g); § 6662(h); § 170(f)(11); § 6707A; Circular 230 § 10.50",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6695AInput {
        Section6695AInput {
            misstatement_category: MisstatementCategory::SubstantialValuationMisstatement,
            appraisal_type: AppraisalType::ConservationEasement,
            knew_or_should_have_known_appraisal_use: true,
            underpayment_cents: 1_000_000,
            appraiser_gross_income_cents: 500_000,
            more_likely_than_not_proper_value: false,
            return_filed_after_august_17_2006: true,
            return_filed_after_july_25_2006: true,
        }
    }

    #[test]
    fn substantial_valuation_misstatement_engages_penalty() {
        let r = check(&valid_base());
        assert!(r.penalty_imposed);
        assert!(r.effective_date_satisfied);
    }

    #[test]
    fn no_misstatement_no_penalty() {
        let mut i = valid_base();
        i.misstatement_category = MisstatementCategory::None;
        let r = check(&i);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn estate_gift_understatement_engages_penalty() {
        let mut i = valid_base();
        i.misstatement_category = MisstatementCategory::EstateGiftValuationUnderstatement;
        let r = check(&i);
        assert!(r.penalty_imposed);
    }

    #[test]
    fn gross_valuation_misstatement_engages_penalty() {
        let mut i = valid_base();
        i.misstatement_category = MisstatementCategory::GrossValuationMisstatement;
        let r = check(&i);
        assert!(r.penalty_imposed);
    }

    #[test]
    fn good_faith_more_likely_than_not_exception_blocks_penalty() {
        let mut i = valid_base();
        i.more_likely_than_not_proper_value = true;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert!(r.good_faith_exception_engaged);
    }

    #[test]
    fn appraiser_no_knowledge_no_penalty() {
        let mut i = valid_base();
        i.knew_or_should_have_known_appraisal_use = false;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6695A(a)(2)") && f.contains("KNOWN OR REASONABLY SHOULD HAVE KNOWN")));
    }

    #[test]
    fn return_filed_before_aug_17_2006_no_penalty() {
        let mut i = valid_base();
        i.return_filed_after_august_17_2006 = false;
        let r = check(&i);
        assert!(!r.effective_date_satisfied);
        assert!(!r.penalty_imposed);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("August 17, 2006")));
    }

    #[test]
    fn facade_easement_uses_july_25_2006_effective_date() {
        let mut i = valid_base();
        i.appraisal_type = AppraisalType::FacadeEasementDonation;
        i.return_filed_after_august_17_2006 = false;
        i.return_filed_after_july_25_2006 = true;
        let r = check(&i);
        assert!(r.effective_date_satisfied);
        assert!(r.penalty_imposed);
    }

    #[test]
    fn facade_easement_before_july_25_2006_no_penalty() {
        let mut i = valid_base();
        i.appraisal_type = AppraisalType::FacadeEasementDonation;
        i.return_filed_after_july_25_2006 = false;
        let r = check(&i);
        assert!(!r.effective_date_satisfied);
        assert!(!r.penalty_imposed);
    }

    #[test]
    fn ten_percent_underpayment_greater_than_1000_dollars_used() {
        let mut i = valid_base();
        i.underpayment_cents = 5_000_000;
        i.appraiser_gross_income_cents = 100_000_000;
        let r = check(&i);
        assert_eq!(r.greater_of_amount_cents, 500_000);
        assert_eq!(r.penalty_amount_cents, 500_000);
    }

    #[test]
    fn one_thousand_floor_when_underpayment_low() {
        let mut i = valid_base();
        i.underpayment_cents = 100_000;
        i.appraiser_gross_income_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.greater_of_amount_cents, 100_000);
    }

    #[test]
    fn income_cap_125_percent_caps_penalty() {
        let mut i = valid_base();
        i.underpayment_cents = 100_000_000;
        i.appraiser_gross_income_cents = 200_000;
        let r = check(&i);
        assert_eq!(r.income_cap_cents, 250_000);
        assert_eq!(r.penalty_amount_cents, 250_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6695A(a)-(c)"));
        assert!(r.citation.contains("Pension Protection Act of 2006 § 1219"));
        assert!(r.citation.contains("26 CFR § 1.6695A"));
        assert!(r.citation.contains("IRM 20.1.12"));
        assert!(r.citation.contains("§ 6662(e)"));
        assert!(r.citation.contains("§ 6662(g)"));
        assert!(r.citation.contains("§ 6662(h)"));
        assert!(r.citation.contains("§ 170(f)(11)"));
        assert!(r.citation.contains("§ 6707A"));
        assert!(r.citation.contains("Circular 230 § 10.50"));
    }

    #[test]
    fn note_pins_three_misstatement_categories() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6662(e)")
            && n.contains("§ 6662(g)")
            && n.contains("§ 6662(h)")));
    }

    #[test]
    fn note_pins_penalty_formula_lesser_of() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6695A(b)")
            && n.contains("LESSER OF")
            && n.contains("10%")
            && n.contains("$1,000")
            && n.contains("125%")));
    }

    #[test]
    fn note_pins_good_faith_more_likely_than_not_exception() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6695A(c)")
            && n.contains("MORE LIKELY THAN NOT")
            && n.contains("51%")));
    }

    #[test]
    fn note_pins_substantial_misstatement_150_percent_threshold() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6662(e)")
            && n.contains("150%")));
    }

    #[test]
    fn note_pins_gross_misstatement_200_percent_threshold() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6662(h)")
            && n.contains("200%")
            && n.contains("40%")));
    }

    #[test]
    fn note_pins_estate_gift_65_percent_threshold() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6662(g)")
            && n.contains("65%")));
    }

    #[test]
    fn note_pins_august_17_2006_effective_date() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("AUGUST 17, 2006")
            && n.contains("JULY 25, 2006")
            && n.contains("Pension Protection Act")));
    }

    #[test]
    fn note_pins_trader_relevant_applications() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("art donations")
            && n.contains("conservation easements")
            && n.contains("facade easements")
            && n.contains("§ 1031")
            && n.contains("syndicated conservation")));
    }

    #[test]
    fn misstatement_category_truth_table_four_cells() {
        for (category, exp_penalty) in [
            (MisstatementCategory::SubstantialValuationMisstatement, true),
            (MisstatementCategory::EstateGiftValuationUnderstatement, true),
            (MisstatementCategory::GrossValuationMisstatement, true),
            (MisstatementCategory::None, false),
        ] {
            let mut i = valid_base();
            i.misstatement_category = category;
            let r = check(&i);
            assert_eq!(
                r.penalty_imposed, exp_penalty,
                "category={:?} expected penalty={}",
                category, exp_penalty
            );
        }
    }

    #[test]
    fn facade_easement_uses_unique_effective_date_invariant() {
        let mut i_facade = valid_base();
        i_facade.appraisal_type = AppraisalType::FacadeEasementDonation;
        i_facade.return_filed_after_august_17_2006 = false;
        i_facade.return_filed_after_july_25_2006 = true;
        let r_facade = check(&i_facade);
        assert!(r_facade.effective_date_satisfied);

        let mut i_general = valid_base();
        i_general.appraisal_type = AppraisalType::GeneralProperty;
        i_general.return_filed_after_august_17_2006 = false;
        i_general.return_filed_after_july_25_2006 = true;
        let r_general = check(&i_general);
        assert!(!r_general.effective_date_satisfied);
    }

    #[test]
    fn good_faith_exception_dominates_invariant() {
        let mut i = valid_base();
        i.misstatement_category = MisstatementCategory::GrossValuationMisstatement;
        i.more_likely_than_not_proper_value = true;
        let r = check(&i);
        assert!(!r.penalty_imposed);
        assert!(r.good_faith_exception_engaged);
    }

    #[test]
    fn penalty_amount_lesser_of_logic_invariant() {
        let mut i_income_caps = valid_base();
        i_income_caps.underpayment_cents = 100_000_000;
        i_income_caps.appraiser_gross_income_cents = 100_000;
        let r_caps = check(&i_income_caps);
        assert_eq!(r_caps.penalty_amount_cents, 125_000);

        let mut i_underpayment_drives = valid_base();
        i_underpayment_drives.underpayment_cents = 1_000_000;
        i_underpayment_drives.appraiser_gross_income_cents = 1_000_000;
        let r_under = check(&i_underpayment_drives);
        assert_eq!(r_under.penalty_amount_cents, 100_000);
    }

    #[test]
    fn defensive_zero_underpayment_uses_1000_floor() {
        let mut i = valid_base();
        i.underpayment_cents = 0;
        i.appraiser_gross_income_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.greater_of_amount_cents, 100_000);
        assert_eq!(r.penalty_amount_cents, 100_000);
    }

    #[test]
    fn defensive_max_u64_income_saturating() {
        let mut i = valid_base();
        i.appraiser_gross_income_cents = u64::MAX;
        let r = check(&i);
        assert!(r.penalty_imposed);
    }
}
