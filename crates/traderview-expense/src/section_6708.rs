//! IRC § 6708 — Failure to maintain lists of advisees
//! with respect to reportable transactions. Direct sibling
//! to section_6707 (material-advisor failure-to-disclose
//! penalty) and section_6112 (list-maintenance
//! substantive obligation). Where § 6707 penalizes failure
//! to FILE Form 8918 disclosure under § 6111, § 6708
//! penalizes failure to MAINTAIN AND PRODUCE the § 6112
//! list of advisees when IRS makes written request. These
//! are TWO INDEPENDENT penalties — same material advisor
//! can be hit with both for same transaction.
//!
//! Enacted by **American Jobs Creation Act of 2004 § 815**
//! (Pub. L. 108-357, **enacted October 22, 2004**) —
//! same statute that created § 6707 + § 6707A as part of
//! anti-shelter penalty regime created in reaction to
//! KPMG / E&Y / BDO Seidman shelter promotion scandals of
//! early 2000s.
//!
//! Trader-critical for material advisors on basket option
//! contracts (Notice 2015-73), conservation easement
//! syndications (Notice 2017-10), micro-captive insurance
//! (Notice 2016-66), § 643 distribution-tier-out trusts,
//! STARS foreign-tax-credit shelters. Per-day penalty
//! UNCAPPED — accumulates rapidly during refusal to
//! produce list. Recent Federal Register agency-collection
//! activity (March 2025) signals continued IRS focus on
//! § 6708 enforcement.
//!
//! **§ 6708(a)(1) Penalty trigger** — if any person who is
//! required to maintain a list under § 6112(a) FAILS to
//! make such list available upon WRITTEN REQUEST to the
//! Secretary in accordance with § 6112(b) WITHIN **20
//! BUSINESS DAYS** after the date of such request, such
//! person SHALL pay a penalty of **$10,000 for each day
//! of such failure AFTER such 20th day**. NO STATUTORY
//! MAXIMUM — penalty accrues daily until list produced.
//!
//! **§ 6708(a)(2) Reasonable cause exception** — no
//! penalty shall be imposed by paragraph (1) with respect
//! to the failure on any day if such failure is due to
//! REASONABLE CAUSE. Distinct from § 6664(d) — § 6708(a)(2)
//! is its own independent reasonable-cause provision and
//! applies regardless of whether transaction is listed or
//! other reportable.
//!
//! **Treas. Reg. § 301.6708-1(c)(3)(ii) Extension
//! requests** — material advisor may request extension of
//! 20-business-day period by providing IRS with:
//! 1. Reason for the requested extension;
//! 2. Period of time required to comply; AND
//! 3. Description of advisor's good-faith effort to
//!    comply within original 20-business-day period.
//!
//! IRS may grant or deny extension request in
//! discretionary judgment.
//!
//! **§ 6112(b) List maintenance requirements (cross-
//! reference, implemented by Treas. Reg. § 301.6112-1)**:
//!
//! Required list content per Treas. Reg. § 301.6112-1(b):
//! 1. **Advisee identifiers**: full legal name + current
//!    mailing address + TIN (Taxpayer Identification
//!    Number);
//! 2. **Transaction identification**: listed or other
//!    category + citation OR § 6111 reportable transaction
//!    number;
//! 3. **Timing**: date the advisee entered the
//!    transaction (if known);
//! 4. **Amount**: amount invested (if known and
//!    reasonably determinable);
//! 5. **Tax treatment**: intended or expected tax
//!    treatment with concise schedule or summary.
//!
//! **List preparation timeline** — material advisor has
//! **30 CALENDAR DAYS** from the date the list-
//! maintenance requirement first arises with respect to a
//! reportable transaction to prepare the list.
//!
//! **List per transaction** — separate list maintained
//! for EACH reportable transaction; ONE list maintained
//! for SUBSTANTIALLY SIMILAR transactions.
//!
//! **§ 6112(b)(2) seven-year retention requirement** —
//! material advisor must maintain each component of list
//! in readily accessible form for **7 YEARS** following
//! EARLIER of:
//! 1. Date material advisor last made tax statement
//!    relating to the transaction; OR
//! 2. Date transaction was last entered into (if known).
//!
//! Coordination with § 6707:
//! - § 6707 penalizes failure to FILE Form 8918 disclosure
//!   under § 6111;
//! - § 6708 penalizes failure to MAINTAIN AND PRODUCE the
//!   § 6112 list of advisees;
//! - Two INDEPENDENT penalties — same material advisor
//!   can be hit with both for same transaction.
//!
//! Citations: 26 USC § 6708(a)(1) + § 6708(a)(2); 26 USC
//! § 6112(a)-(b); American Jobs Creation Act of 2004 § 815
//! (Pub. L. 108-357, October 22, 2004); 26 CFR
//! § 301.6708-1; 26 CFR § 301.6112-1; 26 USC § 6707; 26 USC
//! § 6111; IRM 20.1.13 (Material Advisor and Reportable
//! Transactions Penalties); Form 8918 (Material Advisor
//! Disclosure Statement); Notice 2015-73 (basket option
//! contracts); Notice 2017-10 (conservation easement
//! syndications); Notice 2016-66 (micro-captive insurance).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ListProductionStatus {
    /// List provided within 20 business days of IRS
    /// written request.
    ProvidedWithinTwentyBusinessDays,
    /// List provided after 20 business days but before
    /// IRS imposed penalty.
    ProvidedAfterTwentyBusinessDays,
    /// Material advisor has refused or failed to provide
    /// list (penalty continues to accrue).
    NotProvided,
    /// IRS granted extension under Treas. Reg.
    /// § 301.6708-1(c)(3)(ii); penalty tolled.
    ExtensionGrantedByIrs,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6708Input {
    pub list_production_status: ListProductionStatus,
    /// Business days since IRS written § 6112(b) list
    /// request (only days > 20 count toward penalty).
    pub business_days_since_irs_request: u32,
    /// Whether material advisor has 30-calendar-day
    /// preparation period satisfied per Treas. Reg.
    /// § 301.6112-1.
    pub list_prepared_within_30_calendar_days: bool,
    /// Whether 7-year retention requirement under
    /// § 6112(b)(2) satisfied.
    pub seven_year_retention_satisfied: bool,
    /// Whether list contains all required content
    /// (advisee identifiers + transaction identification
    /// + timing + amount + tax treatment).
    pub list_content_complete: bool,
    /// Whether reasonable cause defense engaged under
    /// § 6708(a)(2).
    pub reasonable_cause_engaged: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6708Result {
    pub list_production_status: ListProductionStatus,
    pub penalty_engaged: bool,
    pub days_subject_to_penalty: u32,
    pub penalty_cents: u64,
    pub reasonable_cause_defense_available: bool,
    pub extension_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6708Input) -> Section6708Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let days_subject_to_penalty = if matches!(
        input.list_production_status,
        ListProductionStatus::ProvidedWithinTwentyBusinessDays
            | ListProductionStatus::ExtensionGrantedByIrs
    ) {
        0
    } else {
        input.business_days_since_irs_request.saturating_sub(20)
    };

    let extension_engaged = matches!(
        input.list_production_status,
        ListProductionStatus::ExtensionGrantedByIrs
    );

    let raw_penalty_cents: u64 = (days_subject_to_penalty as u64).saturating_mul(1_000_000);

    let penalty_cents = if input.reasonable_cause_engaged {
        0
    } else {
        raw_penalty_cents
    };

    let penalty_engaged = penalty_cents > 0;

    if days_subject_to_penalty > 0 && !input.reasonable_cause_engaged && !extension_engaged {
        failure_reasons.push(format!(
            "26 USC § 6708(a)(1) — material advisor required to maintain § 6112 list FAILED to make list available within 20 BUSINESS DAYS of IRS written request; penalty $10,000 PER DAY for each day after 20th day = {} days subject to penalty = ${} cents UNCAPPED",
            days_subject_to_penalty, penalty_cents
        ));
    }

    if input.reasonable_cause_engaged {
        failure_reasons.push(
            "26 USC § 6708(a)(2) — reasonable cause defense engaged; no penalty imposed on any day where failure is due to REASONABLE CAUSE; distinct from § 6664(d) — § 6708(a)(2) is its own independent reasonable-cause provision".to_string(),
        );
    }

    if extension_engaged {
        failure_reasons.push(
            "Treas. Reg. § 301.6708-1(c)(3)(ii) — IRS granted extension of 20-business-day period upon material advisor's request (must provide reason + period required + description of good-faith effort)".to_string(),
        );
    }

    if !input.list_prepared_within_30_calendar_days {
        failure_reasons.push(
            "Treas. Reg. § 301.6112-1 — material advisor has 30 CALENDAR DAYS from date list-maintenance requirement first arises to prepare the list; failure to prepare list within 30 days exposes advisor to § 6708 penalty when IRS request issued".to_string(),
        );
    }

    if !input.seven_year_retention_satisfied {
        failure_reasons.push(
            "26 USC § 6112(b)(2) + Treas. Reg. § 301.6112-1 — material advisor must maintain each component of list in readily accessible form for 7 YEARS following EARLIER of (1) date material advisor last made tax statement relating to transaction OR (2) date transaction was last entered into (if known)".to_string(),
        );
    }

    if !input.list_content_complete {
        failure_reasons.push(
            "Treas. Reg. § 301.6112-1(b) — required list content: (1) advisee identifiers (full legal name + current mailing address + TIN); (2) transaction identification (listed or other category + § 6111 reportable transaction number); (3) timing (date advisee entered transaction); (4) amount invested (if known and reasonably determinable); (5) intended or expected tax treatment".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 6708(a)(1) — if material advisor required to maintain list under § 6112(a) FAILS to make list available upon WRITTEN REQUEST to Secretary in accordance with § 6112(b) within 20 BUSINESS DAYS after date of such request, such person SHALL pay penalty of $10,000 FOR EACH DAY of such failure AFTER such 20th day; NO STATUTORY MAXIMUM".to_string(),
        "26 USC § 6708(a)(2) — reasonable cause exception: no penalty imposed with respect to failure on any day if failure due to REASONABLE CAUSE; distinct from § 6664(d) and applies regardless of whether transaction is listed or other reportable".to_string(),
        "Treas. Reg. § 301.6708-1(c)(3)(ii) — material advisor may request extension of 20-business-day period by providing IRS: (1) reason for requested extension; (2) period of time required to comply; (3) description of advisor's good-faith effort to comply within original 20-business-day period; IRS may grant or deny in discretionary judgment".to_string(),
        "Treas. Reg. § 301.6112-1(b) — required list content: (1) advisee identifiers (full legal name + current mailing address + TIN); (2) transaction identification (listed or other category + § 6111 reportable transaction number); (3) timing (date advisee entered transaction if known); (4) amount invested (if known and reasonably determinable); (5) intended or expected tax treatment (concise schedule or summary)".to_string(),
        "Treas. Reg. § 301.6112-1 — material advisor has 30 CALENDAR DAYS from date list-maintenance requirement first arises with respect to reportable transaction to prepare list; separate list maintained for EACH reportable transaction; ONE list maintained for SUBSTANTIALLY SIMILAR transactions".to_string(),
        "26 USC § 6112(b)(2) + Treas. Reg. § 301.6112-1 — material advisor must maintain each component of list in READILY ACCESSIBLE FORM for 7 YEARS following EARLIER of (1) date material advisor last made tax statement relating to transaction OR (2) date transaction was last entered into (if known)".to_string(),
        "Enacted by American Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22, 2004) — same statute that created § 6707 + § 6707A as part of broader anti-shelter penalty regime in reaction to KPMG / E&Y / BDO Seidman shelter promotion scandals".to_string(),
        "Coordination with § 6707 — § 6707 penalizes failure to FILE Form 8918 disclosure under § 6111; § 6708 penalizes failure to MAINTAIN AND PRODUCE the § 6112 list of advisees; TWO INDEPENDENT PENALTIES — same material advisor can be hit with both for same transaction".to_string(),
        "26 CFR § 301.6708-1 — implementing regulations on Failure to Maintain List of Advisees with Respect to Reportable Transactions; finalized 2016 (T.D. 9762)".to_string(),
        "IRM 20.1.13 (Material Advisor and Reportable Transactions Penalties) — internal IRS administrative guidance on § 6708 + § 6707 + § 6707A + § 6111 + § 6112 enforcement coordination".to_string(),
        "Recent Federal Register agency-collection activity (March 4, 2025, 90 Fed. Reg. 11209) — signals continued IRS focus on § 6708 enforcement and information-collection burden review".to_string(),
        "Listed transaction examples triggering § 6708 list-maintenance obligation: Notice 2015-73 (basket option contracts); Notice 2017-10 (conservation easement syndications); Notice 2016-66 (micro-captive insurance); § 643 distribution-tier-out trusts; STARS foreign-tax-credit shelters".to_string(),
    ];

    Section6708Result {
        list_production_status: input.list_production_status,
        penalty_engaged,
        days_subject_to_penalty,
        penalty_cents,
        reasonable_cause_defense_available: true,
        extension_engaged,
        failure_reasons,
        citation: "26 USC § 6708(a)(1) and § 6708(a)(2); 26 USC § 6112(a)-(b); American Jobs Creation Act of 2004 § 815 (Pub. L. 108-357, October 22, 2004); 26 CFR § 301.6708-1; 26 CFR § 301.6112-1; 26 USC § 6707; 26 USC § 6111; IRM 20.1.13; Form 8918; Notice 2015-73; Notice 2017-10; Notice 2016-66",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn within_20_days_compliant() -> Section6708Input {
        Section6708Input {
            list_production_status: ListProductionStatus::ProvidedWithinTwentyBusinessDays,
            business_days_since_irs_request: 18,
            list_prepared_within_30_calendar_days: true,
            seven_year_retention_satisfied: true,
            list_content_complete: true,
            reasonable_cause_engaged: false,
        }
    }

    #[test]
    fn within_20_days_no_penalty() {
        let r = check(&within_20_days_compliant());
        assert!(!r.penalty_engaged);
        assert_eq!(r.days_subject_to_penalty, 0);
        assert_eq!(r.penalty_cents, 0);
    }

    #[test]
    fn at_day_20_no_penalty_boundary() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::ProvidedAfterTwentyBusinessDays;
        i.business_days_since_irs_request = 20;
        let r = check(&i);
        assert_eq!(r.days_subject_to_penalty, 0);
        assert_eq!(r.penalty_cents, 0);
    }

    #[test]
    fn at_day_21_one_day_penalty_10k() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::ProvidedAfterTwentyBusinessDays;
        i.business_days_since_irs_request = 21;
        let r = check(&i);
        assert_eq!(r.days_subject_to_penalty, 1);
        assert_eq!(r.penalty_cents, 1_000_000);
    }

    #[test]
    fn at_day_30_ten_day_penalty_100k() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::ProvidedAfterTwentyBusinessDays;
        i.business_days_since_irs_request = 30;
        let r = check(&i);
        assert_eq!(r.days_subject_to_penalty, 10);
        assert_eq!(r.penalty_cents, 10_000_000);
    }

    #[test]
    fn at_day_120_one_hundred_day_penalty_1m() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::NotProvided;
        i.business_days_since_irs_request = 120;
        let r = check(&i);
        assert_eq!(r.days_subject_to_penalty, 100);
        assert_eq!(r.penalty_cents, 100_000_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6708(a)(1)")
            && f.contains("$10,000 PER DAY")
            && f.contains("UNCAPPED")));
    }

    #[test]
    fn reasonable_cause_zeros_penalty() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::NotProvided;
        i.business_days_since_irs_request = 200;
        i.reasonable_cause_engaged = true;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6708(a)(2)")
            && f.contains("REASONABLE CAUSE")));
    }

    #[test]
    fn extension_granted_no_penalty() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::ExtensionGrantedByIrs;
        i.business_days_since_irs_request = 100;
        let r = check(&i);
        assert_eq!(r.penalty_cents, 0);
        assert!(r.extension_engaged);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 301.6708-1(c)(3)(ii)")
            && f.contains("granted extension")));
    }

    #[test]
    fn missing_30_day_prep_violation() {
        let mut i = within_20_days_compliant();
        i.list_prepared_within_30_calendar_days = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 301.6112-1")
            && f.contains("30 CALENDAR DAYS")));
    }

    #[test]
    fn missing_7_year_retention_violation() {
        let mut i = within_20_days_compliant();
        i.seven_year_retention_satisfied = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6112(b)(2)")
            && f.contains("7 YEARS")));
    }

    #[test]
    fn missing_content_violation() {
        let mut i = within_20_days_compliant();
        i.list_content_complete = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 301.6112-1(b)")
            && f.contains("advisee identifiers")
            && f.contains("transaction identification")
            && f.contains("TIN")));
    }

    #[test]
    fn list_production_status_truth_table_four_cells() {
        for (status, days, exp_penalty) in [
            (ListProductionStatus::ProvidedWithinTwentyBusinessDays, 18, 0_u64),
            (ListProductionStatus::ProvidedAfterTwentyBusinessDays, 30, 10_000_000),
            (ListProductionStatus::NotProvided, 50, 30_000_000),
            (ListProductionStatus::ExtensionGrantedByIrs, 100, 0),
        ] {
            let mut i = within_20_days_compliant();
            i.list_production_status = status;
            i.business_days_since_irs_request = days;
            let r = check(&i);
            assert_eq!(r.penalty_cents, exp_penalty, "status={:?} days={}", status, days);
        }
    }

    #[test]
    fn penalty_uncapped_progressive_invariant() {
        let make = |days| {
            let mut i = within_20_days_compliant();
            i.list_production_status = ListProductionStatus::NotProvided;
            i.business_days_since_irs_request = days;
            check(&i)
        };
        let day_50 = make(50);
        let day_100 = make(100);
        let day_365 = make(365);
        let day_1000 = make(1000);
        assert!(day_50.penalty_cents < day_100.penalty_cents);
        assert!(day_100.penalty_cents < day_365.penalty_cents);
        assert!(day_365.penalty_cents < day_1000.penalty_cents);
        assert_eq!(day_50.penalty_cents, 30_000_000);
        assert_eq!(day_100.penalty_cents, 80_000_000);
        assert_eq!(day_365.penalty_cents, 345_000_000);
        assert_eq!(day_1000.penalty_cents, 980_000_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&within_20_days_compliant());
        assert!(r.citation.contains("§ 6708(a)(1)"));
        assert!(r.citation.contains("§ 6708(a)(2)"));
        assert!(r.citation.contains("§ 6112(a)-(b)"));
        assert!(r.citation.contains("American Jobs Creation Act of 2004 § 815"));
        assert!(r.citation.contains("Pub. L. 108-357"));
        assert!(r.citation.contains("October 22, 2004"));
        assert!(r.citation.contains("26 CFR § 301.6708-1"));
        assert!(r.citation.contains("26 CFR § 301.6112-1"));
        assert!(r.citation.contains("§ 6707"));
        assert!(r.citation.contains("§ 6111"));
        assert!(r.citation.contains("IRM 20.1.13"));
        assert!(r.citation.contains("Form 8918"));
        assert!(r.citation.contains("Notice 2015-73"));
        assert!(r.citation.contains("Notice 2017-10"));
        assert!(r.citation.contains("Notice 2016-66"));
    }

    #[test]
    fn note_pins_subsection_a1_20_day_trigger_10k_per_day() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 6708(a)(1)")
            && n.contains("20 BUSINESS DAYS")
            && n.contains("$10,000 FOR EACH DAY")
            && n.contains("NO STATUTORY MAXIMUM")));
    }

    #[test]
    fn note_pins_subsection_a2_reasonable_cause() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 6708(a)(2)")
            && n.contains("REASONABLE CAUSE")
            && n.contains("§ 6664(d)")));
    }

    #[test]
    fn note_pins_extension_request_treas_reg_301_6708_1() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.6708-1(c)(3)(ii)")
            && n.contains("reason for requested extension")
            && n.contains("good-faith effort")));
    }

    #[test]
    fn note_pins_list_content_five_elements() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.6112-1(b)")
            && n.contains("advisee identifiers")
            && n.contains("TIN")
            && n.contains("transaction identification")
            && n.contains("amount invested")
            && n.contains("tax treatment")));
    }

    #[test]
    fn note_pins_30_calendar_day_preparation() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.6112-1")
            && n.contains("30 CALENDAR DAYS")
            && n.contains("separate list")
            && n.contains("SUBSTANTIALLY SIMILAR transactions")));
    }

    #[test]
    fn note_pins_7_year_retention() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 6112(b)(2)")
            && n.contains("READILY ACCESSIBLE FORM")
            && n.contains("7 YEARS")));
    }

    #[test]
    fn note_pins_2004_ajca_origin() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("American Jobs Creation Act of 2004 § 815")
            && n.contains("Pub. L. 108-357")
            && n.contains("October 22, 2004")
            && n.contains("KPMG")));
    }

    #[test]
    fn note_pins_coordination_with_6707() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Coordination with § 6707")
            && n.contains("TWO INDEPENDENT PENALTIES")
            && n.contains("same material advisor can be hit with both")));
    }

    #[test]
    fn note_pins_301_6708_1_implementing_regulations() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("26 CFR § 301.6708-1")
            && n.contains("T.D. 9762")));
    }

    #[test]
    fn note_pins_irm_20_1_13() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("IRM 20.1.13")
            && n.contains("§ 6707")
            && n.contains("§ 6707A")
            && n.contains("§ 6112")));
    }

    #[test]
    fn note_pins_2025_federal_register_continued_enforcement() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Federal Register agency-collection activity")
            && n.contains("March 4, 2025")
            && n.contains("continued IRS focus")));
    }

    #[test]
    fn note_pins_listed_transaction_examples() {
        let r = check(&within_20_days_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Listed transaction examples")
            && n.contains("Notice 2015-73")
            && n.contains("Notice 2017-10")
            && n.contains("Notice 2016-66")));
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::NotProvided;
        i.business_days_since_irs_request = u32::MAX;
        let r = check(&i);
        let _ = r.penalty_cents;
        assert!(r.penalty_engaged);
    }

    #[test]
    fn multiple_failure_reasons_stack() {
        let mut i = within_20_days_compliant();
        i.list_production_status = ListProductionStatus::NotProvided;
        i.business_days_since_irs_request = 100;
        i.list_prepared_within_30_calendar_days = false;
        i.seven_year_retention_satisfied = false;
        i.list_content_complete = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }

    #[test]
    fn reasonable_cause_uniquely_zeros_penalty_invariant() {
        let mut without_rc = within_20_days_compliant();
        without_rc.list_production_status = ListProductionStatus::NotProvided;
        without_rc.business_days_since_irs_request = 50;
        let r_without = check(&without_rc);
        assert!(r_without.penalty_cents > 0);

        let mut with_rc = within_20_days_compliant();
        with_rc.list_production_status = ListProductionStatus::NotProvided;
        with_rc.business_days_since_irs_request = 50;
        with_rc.reasonable_cause_engaged = true;
        let r_with = check(&with_rc);
        assert_eq!(r_with.penalty_cents, 0);
    }

    #[test]
    fn extension_distinct_from_reasonable_cause_invariant() {
        let mut extension = within_20_days_compliant();
        extension.list_production_status = ListProductionStatus::ExtensionGrantedByIrs;
        extension.business_days_since_irs_request = 100;
        let r_ext = check(&extension);
        assert!(r_ext.extension_engaged);
        // `reasonable_cause_defense_available` is independent of the extension
        // path in this fixture — the invariant pinned here is the extension
        // flag, not the reasonable-cause flag.

        let mut rc_only = within_20_days_compliant();
        rc_only.list_production_status = ListProductionStatus::NotProvided;
        rc_only.business_days_since_irs_request = 100;
        rc_only.reasonable_cause_engaged = true;
        let r_rc = check(&rc_only);
        assert!(!r_rc.extension_engaged);
    }
}
