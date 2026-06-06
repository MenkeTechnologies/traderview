//! IRC § 6303 — Notice and demand for tax. Foundational
//! procedural requirement for IRS collection. Cross-
//! referenced by § 6321 (lien attachment requires notice
//! and demand) + § 6331 (levy authority requires notice and
//! demand + 10-day neglect). Trader-relevant because no
//! lawful IRS lien, levy, or seizure may proceed without
//! proper § 6303 notice and demand. Procedural-companion to
//! § 6321, § 6323, § 6325, § 6331, § 6334 (levy/lien
//! constellation), § 6212 (SNOD), § 6213 (Tax Court
//! petition), § 6601 (interest on unpaid tax).
//!
//! **§ 6303(a) general rule**: Secretary shall, as soon as
//! practicable and within **60 days** after the making of
//! an assessment of tax under § 6203, give notice to each
//! person liable for the unpaid tax, stating:
//! 1. the **amount** of unpaid tax, AND
//! 2. **demanding payment** thereof.
//!
//! **§ 6303(a) manner of delivery** — notice shall be:
//! 1. left at the dwelling of such person, OR
//! 2. left at the usual place of business of such person,
//!    OR
//! 3. sent by mail to such person's last known address.
//!
//! Certified mail is NOT required by § 6303(a); ordinary
//! mail to last known address suffices.
//!
//! **§ 6303(a) — failure to give notice within 60 days does
//! NOT invalidate notice**. Late notice remains valid;
//! collection authority engages once notice and demand are
//! properly delivered.
//!
//! **§ 6303(b) — assessment prior to last date for payment**:
//! if tax is assessed BEFORE the last date prescribed for
//! payment, payment shall not be demanded under § 6303(a)
//! until AFTER such date — EXCEPT where the Secretary
//! believes collection would be jeopardized by delay
//! (§ 6861/§ 6862 jeopardy + § 7429 review).
//!
//! Citations: 26 USC § 6303(a)-(b); 26 CFR § 301.6303-1;
//! § 6203 (method of assessment); § 6321 (lien arising); §
//! 6331 (levy authority); § 6212 (SNOD); § 6213 (Tax Court
//! petition); § 6601 (interest); § 6861/§ 6862 (jeopardy
//! assessment); § 7429 (jeopardy review).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryManner {
    /// § 6303(a)(1) — left at dwelling.
    LeftAtDwelling,
    /// § 6303(a)(2) — left at usual place of business.
    LeftAtPlaceOfBusiness,
    /// § 6303(a)(3) — sent by mail to last known address.
    MailedToLastKnownAddress,
    /// Notice not delivered at all.
    NoDelivery,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6303Input {
    /// Days since assessment was made under § 6203.
    pub days_since_assessment: u32,
    /// Whether notice was given.
    pub notice_given: bool,
    /// Manner of delivery used.
    pub delivery_manner: DeliveryManner,
    /// Whether notice states amount of unpaid tax.
    pub amount_stated: bool,
    /// Whether notice demands payment.
    pub payment_demanded: bool,
    /// Whether tax was assessed BEFORE last date prescribed
    /// for payment (triggers § 6303(b) delayed-demand rule).
    pub assessed_before_payment_due_date: bool,
    /// Whether the payment due date has now passed.
    pub payment_due_date_passed: bool,
    /// Whether Secretary believes collection is jeopardized
    /// (bypasses § 6303(b) delayed-demand rule).
    pub jeopardy_finding: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6303Result {
    pub notice_and_demand_valid: bool,
    pub within_60_day_window: bool,
    pub delayed_demand_rule_engaged: bool,
    pub demand_may_be_made_now: bool,
    pub jeopardy_bypass: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6303Input) -> Section6303Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.notice_given {
        failure_reasons.push(
            "26 USC § 6303(a) — notice and demand must be given before lien attaches (§ 6321) or levy may be made (§ 6331)"
                .to_string(),
        );
    }

    let valid_delivery = !matches!(input.delivery_manner, DeliveryManner::NoDelivery);

    if input.notice_given && !valid_delivery {
        failure_reasons.push(
            "26 USC § 6303(a) — notice must be (1) left at dwelling, (2) left at usual place of business, or (3) sent by mail to last known address".to_string(),
        );
    }

    if input.notice_given && !input.amount_stated {
        failure_reasons
            .push("26 USC § 6303(a) — notice must STATE THE AMOUNT of unpaid tax".to_string());
    }

    if input.notice_given && !input.payment_demanded {
        failure_reasons
            .push("26 USC § 6303(a) — notice must DEMAND PAYMENT of unpaid tax".to_string());
    }

    let within_60_day = input.days_since_assessment <= 60;

    let delayed_demand_engaged = input.assessed_before_payment_due_date
        && !input.payment_due_date_passed
        && !input.jeopardy_finding;

    if delayed_demand_engaged {
        failure_reasons.push(
            "26 USC § 6303(b) — payment may not be demanded until AFTER last date prescribed for payment when tax assessed before due date (except where Secretary finds collection jeopardized by delay)".to_string(),
        );
    }

    let demand_may_be_made_now = !delayed_demand_engaged;

    let notes: Vec<String> = vec![
        "26 USC § 6303(a) — Secretary shall, as soon as practicable and within 60 days after assessment under § 6203, give notice to each person liable for unpaid tax stating amount and demanding payment"
            .to_string(),
        "26 USC § 6303(a) manner of delivery — (1) left at dwelling, (2) left at usual place of business, or (3) sent by mail to last known address; certified mail NOT required"
            .to_string(),
        "26 USC § 6303(a) — failure to give notice within 60 days does NOT invalidate notice; late notice remains valid; collection authority engages once notice and demand are properly delivered"
            .to_string(),
        "26 USC § 6303(b) — if tax assessed BEFORE last date prescribed for payment, demand shall not be made until AFTER such date (except jeopardy finding under § 6861/§ 6862 with § 7429 review)"
            .to_string(),
        "Cross-references: § 6303 notice and demand is foundational predicate for § 6321 lien attachment + § 6331 levy authority (10-day neglect rule begins after notice and demand)"
            .to_string(),
    ];

    Section6303Result {
        notice_and_demand_valid: failure_reasons.is_empty() && input.notice_given,
        within_60_day_window: within_60_day,
        delayed_demand_rule_engaged: delayed_demand_engaged,
        demand_may_be_made_now,
        jeopardy_bypass: input.jeopardy_finding,
        failure_reasons,
        citation: "26 USC § 6303(a)-(b); 26 CFR § 301.6303-1; § 6203; § 6321; § 6331; § 6212; § 6213; § 6601; § 6861; § 6862; § 7429",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6303Input {
        Section6303Input {
            days_since_assessment: 30,
            notice_given: true,
            delivery_manner: DeliveryManner::MailedToLastKnownAddress,
            amount_stated: true,
            payment_demanded: true,
            assessed_before_payment_due_date: false,
            payment_due_date_passed: true,
            jeopardy_finding: false,
        }
    }

    #[test]
    fn fully_compliant_within_60_day_window() {
        let r = check(&valid_base());
        assert!(r.notice_and_demand_valid);
        assert!(r.within_60_day_window);
        assert!(r.demand_may_be_made_now);
        assert!(!r.delayed_demand_rule_engaged);
    }

    #[test]
    fn at_60_day_boundary_valid() {
        let mut i = valid_base();
        i.days_since_assessment = 60;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(r.within_60_day_window);
    }

    #[test]
    fn at_61_days_still_valid_but_outside_60_day_window() {
        let mut i = valid_base();
        i.days_since_assessment = 61;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(!r.within_60_day_window);
    }

    #[test]
    fn no_notice_given_invalid() {
        let mut i = valid_base();
        i.notice_given = false;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6303(a)") && f.contains("notice and demand must be given")));
    }

    #[test]
    fn no_delivery_invalid() {
        let mut i = valid_base();
        i.delivery_manner = DeliveryManner::NoDelivery;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6303(a)") && f.contains("dwelling")));
    }

    #[test]
    fn left_at_dwelling_valid() {
        let mut i = valid_base();
        i.delivery_manner = DeliveryManner::LeftAtDwelling;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
    }

    #[test]
    fn left_at_place_of_business_valid() {
        let mut i = valid_base();
        i.delivery_manner = DeliveryManner::LeftAtPlaceOfBusiness;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
    }

    #[test]
    fn mailed_to_last_known_address_valid() {
        let mut i = valid_base();
        i.delivery_manner = DeliveryManner::MailedToLastKnownAddress;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
    }

    #[test]
    fn no_amount_stated_invalid() {
        let mut i = valid_base();
        i.amount_stated = false;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("STATE THE AMOUNT")));
    }

    #[test]
    fn no_payment_demanded_invalid() {
        let mut i = valid_base();
        i.payment_demanded = false;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("DEMAND PAYMENT")));
    }

    #[test]
    fn delayed_demand_engaged_when_assessed_before_due_date() {
        let mut i = valid_base();
        i.assessed_before_payment_due_date = true;
        i.payment_due_date_passed = false;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert!(r.delayed_demand_rule_engaged);
        assert!(!r.demand_may_be_made_now);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6303(b)") && f.contains("AFTER last date")));
    }

    #[test]
    fn delayed_demand_bypassed_by_jeopardy_finding() {
        let mut i = valid_base();
        i.assessed_before_payment_due_date = true;
        i.payment_due_date_passed = false;
        i.jeopardy_finding = true;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(!r.delayed_demand_rule_engaged);
        assert!(r.jeopardy_bypass);
    }

    #[test]
    fn delayed_demand_not_engaged_when_due_date_passed() {
        let mut i = valid_base();
        i.assessed_before_payment_due_date = true;
        i.payment_due_date_passed = true;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(!r.delayed_demand_rule_engaged);
    }

    #[test]
    fn delayed_demand_not_engaged_when_assessed_after_due_date() {
        let mut i = valid_base();
        i.assessed_before_payment_due_date = false;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(!r.delayed_demand_rule_engaged);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6303(a)-(b)"));
        assert!(r.citation.contains("§ 301.6303-1"));
        assert!(r.citation.contains("§ 6203"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6331"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("§ 6213"));
        assert!(r.citation.contains("§ 6601"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6862"));
        assert!(r.citation.contains("§ 7429"));
    }

    #[test]
    fn note_pins_60_day_rule() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6303(a)") && n.contains("60 days") && n.contains("§ 6203")));
    }

    #[test]
    fn note_pins_three_delivery_methods() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("dwelling")
            && n.contains("usual place of business")
            && n.contains("last known address")
            && n.contains("certified mail NOT required")));
    }

    #[test]
    fn note_pins_late_notice_valid() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("NOT invalidate") && n.contains("late notice")));
    }

    #[test]
    fn note_pins_delayed_demand_jeopardy_exception() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6303(b)") && n.contains("§ 6861") && n.contains("§ 7429")));
    }

    #[test]
    fn note_pins_cross_reference_predicate_for_lien_and_levy() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6321") && n.contains("§ 6331") && n.contains("predicate")));
    }

    #[test]
    fn delivery_manner_truth_table() {
        for (manner, exp_valid) in [
            (DeliveryManner::LeftAtDwelling, true),
            (DeliveryManner::LeftAtPlaceOfBusiness, true),
            (DeliveryManner::MailedToLastKnownAddress, true),
            (DeliveryManner::NoDelivery, false),
        ] {
            let mut i = valid_base();
            i.delivery_manner = manner;
            let r = check(&i);
            assert_eq!(r.notice_and_demand_valid, exp_valid);
        }
    }

    #[test]
    fn delayed_demand_truth_table() {
        for (assessed_before, due_passed, jeopardy, exp_engaged) in [
            (false, false, false, false),
            (false, true, false, false),
            (true, false, false, true),
            (true, true, false, false),
            (true, false, true, false),
            (true, true, true, false),
            (false, false, true, false),
        ] {
            let mut i = valid_base();
            i.assessed_before_payment_due_date = assessed_before;
            i.payment_due_date_passed = due_passed;
            i.jeopardy_finding = jeopardy;
            let r = check(&i);
            assert_eq!(r.delayed_demand_rule_engaged, exp_engaged);
        }
    }

    #[test]
    fn multiple_content_failures_stack() {
        let mut i = valid_base();
        i.amount_stated = false;
        i.payment_demanded = false;
        let r = check(&i);
        assert!(!r.notice_and_demand_valid);
        assert_eq!(r.failure_reasons.len(), 2);
    }

    #[test]
    fn sixty_day_satisfied_with_late_notice_still_valid() {
        let mut i = valid_base();
        i.days_since_assessment = 365;
        let r = check(&i);
        assert!(r.notice_and_demand_valid);
        assert!(!r.within_60_day_window);
    }

    #[test]
    fn jeopardy_bypass_truth_table() {
        for (jeopardy, exp_bypass) in [(false, false), (true, true)] {
            let mut i = valid_base();
            i.jeopardy_finding = jeopardy;
            let r = check(&i);
            assert_eq!(r.jeopardy_bypass, exp_bypass);
        }
    }

    #[test]
    fn assessment_after_due_date_no_delayed_demand_invariant() {
        let mut i_after = valid_base();
        i_after.assessed_before_payment_due_date = false;
        let r_after = check(&i_after);
        assert!(!r_after.delayed_demand_rule_engaged);

        let mut i_before = valid_base();
        i_before.assessed_before_payment_due_date = true;
        i_before.payment_due_date_passed = false;
        let r_before = check(&i_before);
        assert!(r_before.delayed_demand_rule_engaged);
    }
}
