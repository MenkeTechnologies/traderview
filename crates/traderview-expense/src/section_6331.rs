//! IRC § 6331 — Levy and distraint authority. Foundational
//! IRS levy statute. Trader-relevant for any taxpayer facing
//! IRS levy threat: trader-traders with unpaid tax can lose
//! brokerage accounts; trader-landlords with unpaid tax can
//! lose rental income via continuous levy. Pairs with §
//! 6321 (lien attachment), § 6323 (lien priority), § 6325
//! (release), § 6334 (exempt property), and § 6330 (CDP for
//! levies).
//!
//! Procedural-companion to § 6321 (lien attachment), § 6323
//! (lien priority), § 6325 (release/discharge), § 6330 (CDP
//! for levies), § 6334 (exempt property), § 7421 (Anti-
//! Injunction Act), § 7426 (third-party wrongful levy),
//! § 7429 (jeopardy review), § 7433 (civil damages for
//! unauthorized collection), and § 7508A (disaster
//! postponement).
//!
//! **§ 6331(a) levy authority** — Secretary may levy upon
//! property of taxpayer who has failed to pay tax within
//! **10 days after notice and demand** under § 6303.
//!
//! **§ 6331(d) 30-day pre-levy notice** — Secretary must
//! provide taxpayer 30-day notice of intent to levy under
//! § 6331(d)(2); notice may be (a) given in person, (b)
//! left at dwelling or usual place of business, or (c)
//! sent by certified or registered mail to last known
//! address.
//!
//! **§ 6331(e) continuous wage levy** — levy on salary or
//! wages has continuous effect from time of levy until
//! release. Attaches to: (1) wages earned but not yet paid;
//! (2) advances subsequent to levy; (3) wages becoming
//! payable subsequent to levy.
//!
//! **§ 6331(h) continuous levy on specific federal payments**
//! — up to 15% of federal payments (Social Security, federal
//! employee retirement, etc.) subject to continuous levy.
//!
//! **§ 6331(i) bond proceeds exception** — proceeds from
//! bonds purchased pursuant to authority of § 6311 are not
//! subject to levy until issuance complete; uniformed
//! services bond exemption.
//!
//! **§ 6331(j) jeopardy levy exception** — § 6331(d) 30-day
//! pre-levy notice does NOT apply if Secretary finds that
//! collection of tax is in jeopardy (parallel § 6861/§ 6862
//! jeopardy assessment framework with § 7429 judicial
//! review).
//!
//! **§ 6331(k)(1) innocent spouse pending** — no levy while
//! innocent spouse relief request under § 6015 pending.
//!
//! **§ 6331(k)(2) CDP request pending** — no levy while
//! Collection Due Process hearing under § 6330 pending.
//!
//! Citations: 26 USC § 6331(a)-(k); 26 CFR § 301.6331-1; IRM
//! 5.17.3 (Levy and Sale); § 6303 (notice and demand); § 6311
//! (acceptance of bonds); § 6330 (CDP); § 6015 (innocent
//! spouse); § 6861/§ 6862 (jeopardy assessment); § 7429
//! (jeopardy review).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LevyType {
    /// § 6331(a) general property levy.
    GeneralProperty,
    /// § 6331(e) continuous wage levy.
    ContinuousWageLevy,
    /// § 6331(h) continuous federal payment levy (15% cap).
    ContinuousFederalPayment,
    /// § 6331(j) jeopardy levy (no pre-levy notice required).
    JeopardyLevy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6331Input {
    pub levy_type: LevyType,
    /// Whether § 6303 notice and demand has been issued.
    pub notice_and_demand_issued: bool,
    /// Days since notice and demand (for § 6331(a) 10-day
    /// neglect period).
    pub days_since_notice_and_demand: u32,
    /// Whether taxpayer has neglected or refused to pay
    /// within 10 days.
    pub taxpayer_neglected_to_pay: bool,
    /// Whether § 6331(d) 30-day pre-levy notice has been
    /// provided.
    pub pre_levy_30_day_notice_provided: bool,
    /// Whether innocent spouse relief request under § 6015
    /// is pending.
    pub innocent_spouse_pending: bool,
    /// Whether CDP hearing request under § 6330 is pending.
    pub cdp_request_pending: bool,
    /// Whether IRS has made a jeopardy finding (for §
    /// 6331(j) exception to 30-day notice).
    pub jeopardy_finding_made: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6331Result {
    pub levy_authorized: bool,
    pub ten_day_neglect_period_satisfied: bool,
    pub thirty_day_notice_required: bool,
    pub thirty_day_notice_satisfied: bool,
    pub continuous_effect_engaged: bool,
    pub bypass_30_day_notice: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6331Input) -> Section6331Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    if !input.notice_and_demand_issued {
        failure_reasons.push(
            "26 USC § 6303 + § 6331(a) — notice and demand for tax must be issued before levy authority engages"
                .to_string(),
        );
    }

    let ten_day_satisfied = input.notice_and_demand_issued
        && input.days_since_notice_and_demand >= 10
        && input.taxpayer_neglected_to_pay;

    if input.notice_and_demand_issued && !ten_day_satisfied {
        failure_reasons.push(format!(
            "26 USC § 6331(a) — taxpayer must neglect or refuse to pay within 10 days after notice and demand ({} days elapsed; taxpayer neglected: {})",
            input.days_since_notice_and_demand, input.taxpayer_neglected_to_pay
        ));
    }

    let jeopardy_bypass =
        matches!(input.levy_type, LevyType::JeopardyLevy) || input.jeopardy_finding_made;

    let thirty_day_required = !jeopardy_bypass;

    if thirty_day_required && !input.pre_levy_30_day_notice_provided {
        failure_reasons.push(
            "26 USC § 6331(d) — 30-day pre-levy notice required (in person, dwelling/place of business, or certified/registered mail to last known address)".to_string(),
        );
    }

    if input.innocent_spouse_pending {
        failure_reasons.push(
            "26 USC § 6331(k)(1) — no levy while innocent spouse relief request under § 6015 pending".to_string(),
        );
    }

    if input.cdp_request_pending {
        failure_reasons.push(
            "26 USC § 6331(k)(2) — no levy while Collection Due Process hearing under § 6330 pending".to_string(),
        );
    }

    let continuous_effect = matches!(
        input.levy_type,
        LevyType::ContinuousWageLevy | LevyType::ContinuousFederalPayment
    );

    let notes: Vec<String> = vec![
        "26 USC § 6331(a) — Secretary may levy upon property of taxpayer who neglects or refuses to pay tax within 10 days after notice and demand under § 6303"
            .to_string(),
        "26 USC § 6331(d) — 30-day pre-levy notice required (in person, dwelling/place of business, or certified/registered mail to last known address); paired with § 6330 CDP framework"
            .to_string(),
        "26 USC § 6331(e) continuous wage levy — attaches to (1) wages earned but not yet paid; (2) advances subsequent to levy; (3) wages becoming payable subsequent to levy; continues until released"
            .to_string(),
        "26 USC § 6331(h) — continuous levy on up to 15% of specified federal payments (Social Security + federal employee retirement + etc.)"
            .to_string(),
        "26 USC § 6331(j) jeopardy levy exception — 30-day pre-levy notice DOES NOT apply if Secretary finds collection in jeopardy; paired with § 6861/§ 6862 jeopardy assessment + § 7429 judicial review"
            .to_string(),
        "26 USC § 6331(k) — no levy while (1) innocent spouse relief request under § 6015 pending OR (2) CDP hearing under § 6330 pending"
            .to_string(),
    ];

    Section6331Result {
        levy_authorized: failure_reasons.is_empty(),
        ten_day_neglect_period_satisfied: ten_day_satisfied,
        thirty_day_notice_required: thirty_day_required,
        thirty_day_notice_satisfied: input.pre_levy_30_day_notice_provided,
        continuous_effect_engaged: continuous_effect,
        bypass_30_day_notice: jeopardy_bypass,
        failure_reasons,
        citation: "26 USC § 6331(a)-(k); 26 CFR § 301.6331-1; IRM 5.17.3; § 6303; § 6311; § 6330; § 6015; § 6861; § 6862; § 7429",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authorized_base() -> Section6331Input {
        Section6331Input {
            levy_type: LevyType::GeneralProperty,
            notice_and_demand_issued: true,
            days_since_notice_and_demand: 30,
            taxpayer_neglected_to_pay: true,
            pre_levy_30_day_notice_provided: true,
            innocent_spouse_pending: false,
            cdp_request_pending: false,
            jeopardy_finding_made: false,
        }
    }

    #[test]
    fn fully_compliant_general_property_levy_authorized() {
        let r = check(&authorized_base());
        assert!(r.levy_authorized);
        assert!(r.ten_day_neglect_period_satisfied);
        assert!(r.thirty_day_notice_required);
        assert!(r.thirty_day_notice_satisfied);
        assert!(!r.continuous_effect_engaged);
        assert!(!r.bypass_30_day_notice);
    }

    #[test]
    fn no_notice_and_demand_violates() {
        let mut i = authorized_base();
        i.notice_and_demand_issued = false;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6303") && f.contains("notice and demand")));
    }

    #[test]
    fn within_10_day_window_violates() {
        let mut i = authorized_base();
        i.days_since_notice_and_demand = 9;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(!r.ten_day_neglect_period_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6331(a)") && f.contains("10 days") && f.contains("9")));
    }

    #[test]
    fn at_10_day_boundary_authorized() {
        let mut i = authorized_base();
        i.days_since_notice_and_demand = 10;
        let r = check(&i);
        assert!(r.levy_authorized);
    }

    #[test]
    fn no_neglect_to_pay_violates() {
        let mut i = authorized_base();
        i.taxpayer_neglected_to_pay = false;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6331(a)") && f.contains("neglected: false")));
    }

    #[test]
    fn no_30_day_notice_violates() {
        let mut i = authorized_base();
        i.pre_levy_30_day_notice_provided = false;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6331(d)") && f.contains("30-day pre-levy notice")));
    }

    #[test]
    fn jeopardy_levy_bypasses_30_day_notice() {
        let mut i = authorized_base();
        i.levy_type = LevyType::JeopardyLevy;
        i.pre_levy_30_day_notice_provided = false;
        let r = check(&i);
        assert!(r.levy_authorized);
        assert!(r.bypass_30_day_notice);
        assert!(!r.thirty_day_notice_required);
    }

    #[test]
    fn jeopardy_finding_bypasses_30_day_notice() {
        let mut i = authorized_base();
        i.jeopardy_finding_made = true;
        i.pre_levy_30_day_notice_provided = false;
        let r = check(&i);
        assert!(r.levy_authorized);
        assert!(r.bypass_30_day_notice);
    }

    #[test]
    fn innocent_spouse_pending_blocks_levy() {
        let mut i = authorized_base();
        i.innocent_spouse_pending = true;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6331(k)(1)") && f.contains("§ 6015")));
    }

    #[test]
    fn cdp_pending_blocks_levy() {
        let mut i = authorized_base();
        i.cdp_request_pending = true;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6331(k)(2)") && f.contains("§ 6330")));
    }

    #[test]
    fn continuous_wage_levy_engages_continuous_effect() {
        let mut i = authorized_base();
        i.levy_type = LevyType::ContinuousWageLevy;
        let r = check(&i);
        assert!(r.continuous_effect_engaged);
    }

    #[test]
    fn continuous_federal_payment_engages_continuous_effect() {
        let mut i = authorized_base();
        i.levy_type = LevyType::ContinuousFederalPayment;
        let r = check(&i);
        assert!(r.continuous_effect_engaged);
    }

    #[test]
    fn general_property_no_continuous_effect() {
        let r = check(&authorized_base());
        assert!(!r.continuous_effect_engaged);
    }

    #[test]
    fn levy_type_truth_table() {
        for (levy, exp_continuous) in [
            (LevyType::GeneralProperty, false),
            (LevyType::ContinuousWageLevy, true),
            (LevyType::ContinuousFederalPayment, true),
            (LevyType::JeopardyLevy, false),
        ] {
            let mut i = authorized_base();
            i.levy_type = levy;
            let r = check(&i);
            assert_eq!(r.continuous_effect_engaged, exp_continuous);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&authorized_base());
        assert!(r.citation.contains("§ 6331(a)-(k)"));
        assert!(r.citation.contains("§ 301.6331-1"));
        assert!(r.citation.contains("IRM 5.17.3"));
        assert!(r.citation.contains("§ 6303"));
        assert!(r.citation.contains("§ 6311"));
        assert!(r.citation.contains("§ 6330"));
        assert!(r.citation.contains("§ 6015"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6862"));
        assert!(r.citation.contains("§ 7429"));
    }

    #[test]
    fn note_pins_10_day_demand() {
        let r = check(&authorized_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6331(a)") && n.contains("10 days") && n.contains("§ 6303")));
    }

    #[test]
    fn note_pins_30_day_pre_levy_notice() {
        let r = check(&authorized_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6331(d)")
            && n.contains("30-day pre-levy")
            && n.contains("§ 6330")));
    }

    #[test]
    fn note_pins_continuous_wage_levy() {
        let r = check(&authorized_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6331(e)")
            && n.contains("continuous wage levy")
            && n.contains("attaches")));
    }

    #[test]
    fn note_pins_15_percent_federal_payment_cap() {
        let r = check(&authorized_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6331(h)")
            && n.contains("15%")
            && n.contains("Social Security")));
    }

    #[test]
    fn note_pins_jeopardy_levy_exception() {
        let r = check(&authorized_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6331(j)") && n.contains("jeopardy") && n.contains("§ 7429")));
    }

    #[test]
    fn note_pins_k_subsection_blockers() {
        let r = check(&authorized_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6331(k)") && n.contains("§ 6015") && n.contains("§ 6330")));
    }

    #[test]
    fn jeopardy_bypass_truth_table() {
        for (levy, jeopardy_finding, exp_bypass) in [
            (LevyType::GeneralProperty, false, false),
            (LevyType::GeneralProperty, true, true),
            (LevyType::ContinuousWageLevy, false, false),
            (LevyType::ContinuousWageLevy, true, true),
            (LevyType::JeopardyLevy, false, true),
            (LevyType::JeopardyLevy, true, true),
        ] {
            let mut i = authorized_base();
            i.levy_type = levy;
            i.jeopardy_finding_made = jeopardy_finding;
            let r = check(&i);
            assert_eq!(r.bypass_30_day_notice, exp_bypass);
        }
    }

    #[test]
    fn k_subsection_blockers_either_blocks_invariant() {
        let mut i_innocent = authorized_base();
        i_innocent.innocent_spouse_pending = true;
        let r_innocent = check(&i_innocent);
        assert!(!r_innocent.levy_authorized);

        let mut i_cdp = authorized_base();
        i_cdp.cdp_request_pending = true;
        let r_cdp = check(&i_cdp);
        assert!(!r_cdp.levy_authorized);

        let mut i_both = authorized_base();
        i_both.innocent_spouse_pending = true;
        i_both.cdp_request_pending = true;
        let r_both = check(&i_both);
        assert!(!r_both.levy_authorized);
        assert_eq!(r_both.failure_reasons.len(), 2);
    }

    #[test]
    fn multiple_failures_stack_for_bare_levy() {
        let i = Section6331Input {
            levy_type: LevyType::GeneralProperty,
            notice_and_demand_issued: false,
            days_since_notice_and_demand: 0,
            taxpayer_neglected_to_pay: false,
            pre_levy_30_day_notice_provided: false,
            innocent_spouse_pending: false,
            cdp_request_pending: false,
            jeopardy_finding_made: false,
        };
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r.failure_reasons.len() >= 2);
    }

    #[test]
    fn ten_day_uniquely_satisfied_only_with_neglect_invariant() {
        let mut i_neglect = authorized_base();
        i_neglect.days_since_notice_and_demand = 10;
        i_neglect.taxpayer_neglected_to_pay = true;
        let r_neglect = check(&i_neglect);
        assert!(r_neglect.ten_day_neglect_period_satisfied);

        let mut i_no_neglect = authorized_base();
        i_no_neglect.days_since_notice_and_demand = 10;
        i_no_neglect.taxpayer_neglected_to_pay = false;
        let r_no_neglect = check(&i_no_neglect);
        assert!(!r_no_neglect.ten_day_neglect_period_satisfied);
    }

    #[test]
    fn nine_day_violates_ten_day_compliant_precision() {
        let mut i_9 = authorized_base();
        i_9.days_since_notice_and_demand = 9;
        let r_9 = check(&i_9);
        assert!(!r_9.ten_day_neglect_period_satisfied);

        let mut i_10 = authorized_base();
        i_10.days_since_notice_and_demand = 10;
        let r_10 = check(&i_10);
        assert!(r_10.ten_day_neglect_period_satisfied);
    }

    #[test]
    fn cdp_uniquely_pre_emption_invariant() {
        let mut i = authorized_base();
        i.cdp_request_pending = true;
        let r = check(&i);
        assert!(!r.levy_authorized);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Collection Due Process") && f.contains("§ 6330")));
    }
}
