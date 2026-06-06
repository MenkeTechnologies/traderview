//! IRC § 6038C — Information with respect to foreign
//! corporations engaged in U.S. business. The
//! anti-avoidance backstop closing the cluster with
//! section_6038a (25%-foreign-owned DOMESTIC corp / DRE)
//! and section_6038b (transfers TO foreign entities). Where
//! § 6038A reaches DOMESTIC corporations with foreign
//! ownership, § 6038C reaches FOREIGN corporations
//! with U.S. trade-or-business presence — without § 6038C,
//! a foreign corp with U.S. T/B could avoid Form 5472 by
//! structuring around § 6038A's domestic-corp requirement.
//!
//! Statutory origin: Omnibus Budget Reconciliation Act of
//! 1990 § 11315 (Pub. L. 101-508, **enacted November 5,
//! 1990**); effective for information furnished after
//! November 5, 1990 AND records existing on or after
//! March 20, 1990.
//!
//! Trader-critical for international trading structures:
//! - Foreign hedge fund LP with US branch or U.S. office
//!   (US ECI under § 882).
//! - Foreign proprietary trading firm with US-based
//!   traders (creates US trade or business under § 864(b)
//!   absent qualifying for § 864(b)(2) trading safe
//!   harbor).
//! - Foreign trader-managed family office with US
//!   effectively connected income.
//! - Foreign brokerage with US permanent establishment.
//! - **§ 864(b)(2) "trading safe harbor"** provides
//!   important shield for foreign securities/commodities
//!   traders — but losing safe harbor (becoming a dealer,
//!   crossing into hedge-fund-of-funds territory)
//!   triggers § 6038C exposure.
//!
//! **§ 6038C(a) Filing requirement** — if a foreign
//! corporation is engaged in a trade or business within
//! the United States **at any time during a taxable
//! year**, such corporation SHALL furnish at such time and
//! in such manner as Secretary prescribes:
//! 1. Information described in § 6038A(b) (concerning
//!    each related party and reportable transactions); AND
//! 2. Such other information as Secretary requires by
//!    regulations.
//!
//! Foreign corporation must MAINTAIN RECORDS at the
//! location, in the manner, and to the extent prescribed
//! by regulations as may be appropriate to determine the
//! corporation's tax liability.
//!
//! **§ 6038C(b) Penalties — cross-reference to § 6038A** —
//! the penalties of § 6038A apply to ANY FAILURE to
//! furnish required information or maintain records under
//! § 6038C, as if such failure were a failure to comply
//! with § 6038A:
//! 1. **§ 6038A(d)(1) BASE PENALTY $25,000** per taxable
//!    year per reporting corporation;
//! 2. **§ 6038A(d)(2) CONTINUATION PENALTY $25,000** per
//!    30-day period (or fraction) after 90-day IRS
//!    notification — **NO MAXIMUM CAP**;
//! 3. **§ 6038A(d)(3) reasonable cause** (NOT willful
//!    neglect) abatement available.
//!
//! **§ 6038C(c) Limited agent authorization rule** — rules
//! apply to any transaction between reporting corporation
//! and any related party who is a foreign person UNLESS
//! such related party AGREES to authorize the reporting
//! corporation to ACT AS LIMITED AGENT solely for purposes
//! of applying § 7602 (examination of records), § 7603
//! (service of summons), and § 7604 (enforcement of
//! summons) with respect to:
//! 1. Any request by Secretary to examine records or
//!    produce testimony; OR
//! 2. Any summons for such records or testimony.
//!
//! Without limited agent authorization, IRS cannot
//! enforce summons against foreign related party
//! directly → § 6038C imposes the reporting burden on
//! the foreign corp itself.
//!
//! **§ 6038C(d) Definitions cross-reference** — terms
//! "related party", "foreign person", and "records" have
//! same meaning as in § 6038A(c).
//!
//! **§ 864(b)(2) trading safe harbor relevance** —
//! foreign person who is NOT a dealer in stocks or
//! securities and who trades for own account through a
//! resident broker/agent does NOT have a US trade or
//! business; trading must be for the foreign person's
//! own account (not as middleman). If safe harbor
//! qualifies, no § 6038C exposure. If safe harbor lost
//! (dealer activity, brokerage operations, customer
//! activity), § 6038C engages.
//!
//! **§ 6501(c)(8) SOL tolling** — § 6501 assessment SOL
//! does NOT start running until required § 6038C
//! information is filed; non-filing keeps ASED OPEN
//! INDEFINITELY for entire tax year.
//!
//! Citations: 26 USC § 6038C(a)-(d); Omnibus Budget
//! Reconciliation Act of 1990 § 11315 (Pub. L. 101-508,
//! November 5, 1990); 26 USC § 6038A(b)-(e) (cross-
//! referenced penalty framework); 26 USC § 6501(c)(8);
//! 26 USC § 864(b) and § 864(b)(2) (trading safe harbor);
//! 26 USC § 882 (foreign corp ECI taxation); 26 USC § 7602
//! and § 7603 and § 7604 (summons authority); Treas. Reg.
//! § 1.6038A-1 through § 1.6038A-7 (cross-referenced).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsTradeOrBusinessStatus {
    /// Foreign corp engaged in US trade or business at any
    /// time during taxable year.
    EngagedInUsTradeOrBusiness,
    /// Foreign corp qualifies for § 864(b)(2) trading safe
    /// harbor — no US T/B.
    Section864b2TradingSafeHarbor,
    /// Foreign corp has no US T/B activity.
    NoUsTradeOrBusiness,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6038cInput {
    pub us_tb_status: UsTradeOrBusinessStatus,
    /// Number of reportable transactions with related
    /// foreign persons during the taxable year.
    pub reportable_transaction_count: u32,
    /// Whether Form 5472 was filed for the tax year.
    pub form_5472_filed: bool,
    /// Whether required records were maintained at the
    /// location and in the manner prescribed by regulations.
    pub records_maintained: bool,
    /// Days since IRS § 6038A(d)(2) notification of
    /// failure (continuation penalty clock — cross-
    /// referenced to § 6038A).
    pub days_since_irs_notification: u32,
    /// Whether reasonable cause defense applies (cross-
    /// referenced § 6038A(d)(3)).
    pub reasonable_cause_engaged: bool,
    /// Whether related foreign person has agreed to
    /// authorize reporting corp as LIMITED AGENT for
    /// § 7602/§ 7603/§ 7604 summons purposes.
    pub limited_agent_authorization_in_place: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6038cResult {
    pub us_tb_status: UsTradeOrBusinessStatus,
    pub subject_to_section_6038c: bool,
    pub form_5472_filing_required: bool,
    pub base_penalty_cents: u64,
    pub continuation_penalty_cents: u64,
    pub total_penalty_cents: u64,
    pub limited_agent_authorization_satisfied: bool,
    pub section_6501_c8_sol_tolled: bool,
    pub section_864b2_safe_harbor_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6038cInput) -> Section6038cResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let subject_to_section_6038c = matches!(
        input.us_tb_status,
        UsTradeOrBusinessStatus::EngagedInUsTradeOrBusiness
    );

    let form_5472_filing_required =
        subject_to_section_6038c && input.reportable_transaction_count > 0;

    let mut base_penalty_cents: u64 = 0;
    let mut continuation_penalty_cents: u64 = 0;

    if form_5472_filing_required && (!input.form_5472_filed || !input.records_maintained) {
        if !input.reasonable_cause_engaged {
            base_penalty_cents = 2_500_000;
            failure_reasons.push(
                "26 USC § 6038C(b) (cross-reference to § 6038A(d)(1)) — failure of foreign corp engaged in US T/B to file Form 5472 OR maintain § 6038A(b) records triggers $25,000 base penalty per taxable year per reporting corporation".to_string(),
            );

            if input.days_since_irs_notification > 90 {
                let days_beyond_90 = input.days_since_irs_notification.saturating_sub(90);
                let thirty_day_periods = days_beyond_90.div_ceil(30);
                continuation_penalty_cents =
                    2_500_000_u64.saturating_mul(thirty_day_periods as u64);
                failure_reasons.push(format!(
                    "26 USC § 6038C(b) (cross-reference to § 6038A(d)(2)) — continuation penalty $25,000 per 30-day period (or fraction) AFTER 90-day IRS notification — UNCAPPED; {} days past notification = {} thirty-day periods accrued",
                    input.days_since_irs_notification, thirty_day_periods
                ));
            }
        } else {
            failure_reasons.push(
                "26 USC § 6038C(b) (cross-reference to § 6038A(d)(3)) + Treas. Reg. § 1.6038A-4(b) — reasonable cause defense engaged; penalty abatement available if NOT willful neglect".to_string(),
            );
        }
    }

    let limited_agent_authorization_satisfied = !form_5472_filing_required
        || input.limited_agent_authorization_in_place
        || input.records_maintained;

    if form_5472_filing_required && !input.limited_agent_authorization_in_place {
        failure_reasons.push(
            "26 USC § 6038C(c) — without related foreign person AGREEING to authorize reporting corporation as LIMITED AGENT solely for § 7602 (examination) + § 7603 (summons service) + § 7604 (summons enforcement) purposes, IRS cannot enforce summons against foreign related party directly; § 6038C imposes reporting burden on foreign corp itself".to_string(),
        );
    }

    let section_6501_c8_sol_tolled = form_5472_filing_required && !input.form_5472_filed;
    if section_6501_c8_sol_tolled {
        failure_reasons.push(
            "26 USC § 6501(c)(8) — § 6501 assessment SOL does NOT start running until required § 6038C information is filed; non-filing keeps § 6501 ASED OPEN INDEFINITELY for ENTIRE TAX YEAR".to_string(),
        );
    }

    let section_864b2_safe_harbor_engaged = matches!(
        input.us_tb_status,
        UsTradeOrBusinessStatus::Section864b2TradingSafeHarbor
    );

    let total_penalty_cents = base_penalty_cents.saturating_add(continuation_penalty_cents);

    let notes: Vec<String> = vec![
        "26 USC § 6038C(a) — if foreign corporation is engaged in trade or business within the United States at ANY TIME during a taxable year, such corporation SHALL furnish information described in § 6038A(b) (related party + reportable transactions) AND maintain records prescribed by regulations".to_string(),
        "26 USC § 6038C(b) — penalties of § 6038A apply to ANY FAILURE to furnish required information or maintain records under § 6038C as if such failure were a failure to comply with § 6038A: $25,000 base + $25,000/30-day continuation (UNCAPPED after 90-day notification) + reasonable cause defense".to_string(),
        "26 USC § 6038C(c) — rules apply to any transaction between reporting corporation and related party who is a foreign person UNLESS related party AGREES to authorize reporting corp as LIMITED AGENT for § 7602 (examination of records) + § 7603 (service of summons) + § 7604 (enforcement of summons) purposes".to_string(),
        "26 USC § 6038C(d) — terms 'related party', 'foreign person', and 'records' have same meaning as in § 6038A(c) — direct cross-reference, no separate § 6038C definitions".to_string(),
        "26 USC § 864(b) + § 864(b)(2) trading safe harbor — foreign person NOT a dealer in stocks or securities who trades for own account through resident broker/agent does NOT have a US trade or business; trading must be for own account (not as middleman). If safe harbor qualifies, NO § 6038C exposure".to_string(),
        "26 USC § 882 — foreign corporation engaged in US trade or business taxed on income effectively connected with conduct of US trade or business at graduated rates plus branch profits tax under § 884; § 6038C provides the reporting backbone for § 882 compliance".to_string(),
        "Statutory origin: Omnibus Budget Reconciliation Act of 1990 § 11315 (Pub. L. 101-508, enacted November 5, 1990); effective for information furnished after November 5, 1990 AND records existing on or after March 20, 1990".to_string(),
        "26 USC § 6501(c)(8) — § 6501 assessment SOL does NOT start running until required § 6038C information is filed; non-filing keeps § 6501 ASED OPEN INDEFINITELY".to_string(),
        "§ 6038C is the anti-avoidance backstop closing the foreign-corp reporting cluster with § 6038A (25%-foreign-owned DOMESTIC corp / DRE) and § 6038B (transfers TO foreign entities); without § 6038C, foreign corp with US T/B could avoid Form 5472 by structuring around § 6038A domestic-corp requirement".to_string(),
        "Treas. Reg. § 1.6038A-1 through § 1.6038A-7 — § 6038A regulations cross-referenced via § 6038C(d); Form 5472 instructions explicitly cover both § 6038A and § 6038C reporting on same form".to_string(),
    ];

    Section6038cResult {
        us_tb_status: input.us_tb_status,
        subject_to_section_6038c,
        form_5472_filing_required,
        base_penalty_cents,
        continuation_penalty_cents,
        total_penalty_cents,
        limited_agent_authorization_satisfied,
        section_6501_c8_sol_tolled,
        section_864b2_safe_harbor_engaged,
        failure_reasons,
        citation: "26 USC § 6038C(a)-(d); Omnibus Budget Reconciliation Act of 1990 § 11315 (Pub. L. 101-508, November 5, 1990); 26 USC § 6038A(b)-(e); 26 USC § 6501(c)(8); 26 USC § 864(b) and § 864(b)(2); 26 USC § 882; 26 USC § 7602 and § 7603 and § 7604; Treas. Reg. § 1.6038A-1 through § 1.6038A-7; IRS Form 5472; IRM 8.11.5; IRM 20.1.9",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engaged_base() -> Section6038cInput {
        Section6038cInput {
            us_tb_status: UsTradeOrBusinessStatus::EngagedInUsTradeOrBusiness,
            reportable_transaction_count: 3,
            form_5472_filed: true,
            records_maintained: true,
            days_since_irs_notification: 0,
            reasonable_cause_engaged: false,
            limited_agent_authorization_in_place: true,
        }
    }

    #[test]
    fn engaged_compliant_no_penalty() {
        let r = check(&engaged_base());
        assert!(r.subject_to_section_6038c);
        assert!(r.form_5472_filing_required);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn safe_harbor_no_us_tb_no_obligation() {
        let mut i = engaged_base();
        i.us_tb_status = UsTradeOrBusinessStatus::Section864b2TradingSafeHarbor;
        let r = check(&i);
        assert!(!r.subject_to_section_6038c);
        assert!(!r.form_5472_filing_required);
        assert!(r.section_864b2_safe_harbor_engaged);
    }

    #[test]
    fn no_us_tb_no_obligation() {
        let mut i = engaged_base();
        i.us_tb_status = UsTradeOrBusinessStatus::NoUsTradeOrBusiness;
        let r = check(&i);
        assert!(!r.subject_to_section_6038c);
    }

    #[test]
    fn engaged_no_reportable_transactions_no_filing_required() {
        let mut i = engaged_base();
        i.reportable_transaction_count = 0;
        let r = check(&i);
        assert!(r.subject_to_section_6038c);
        assert!(!r.form_5472_filing_required);
    }

    #[test]
    fn failure_to_file_25k_base_penalty() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 2_500_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038C(b)")
            && f.contains("§ 6038A(d)(1)")
            && f.contains("$25,000 base penalty")));
    }

    #[test]
    fn failure_to_maintain_records_25k_base_penalty() {
        let mut i = engaged_base();
        i.records_maintained = false;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 2_500_000);
    }

    #[test]
    fn continuation_penalty_1_period_at_120_days() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 120;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 2_500_000);
        assert_eq!(r.total_penalty_cents, 5_000_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038C(b)")
            && f.contains("§ 6038A(d)(2)")
            && f.contains("UNCAPPED")));
    }

    #[test]
    fn continuation_penalty_3_periods_at_180_days() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 180;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 7_500_000);
        assert_eq!(r.total_penalty_cents, 10_000_000);
    }

    #[test]
    fn continuation_fraction_counts_as_full_period() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 91;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 2_500_000);
    }

    #[test]
    fn no_continuation_at_90_day_boundary() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 90;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 0);
    }

    #[test]
    fn reasonable_cause_zeros_penalty() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 200;
        i.reasonable_cause_engaged = true;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 0);
        assert_eq!(r.continuation_penalty_cents, 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038C(b)")
            && f.contains("§ 6038A(d)(3)")
            && f.contains("reasonable cause")));
    }

    #[test]
    fn limited_agent_authorization_required_when_filing_required() {
        let mut i = engaged_base();
        i.limited_agent_authorization_in_place = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038C(c)")
            && f.contains("LIMITED AGENT")
            && f.contains("§ 7602")
            && f.contains("§ 7603")
            && f.contains("§ 7604")));
    }

    #[test]
    fn limited_agent_authorization_not_required_when_no_filing() {
        let mut i = engaged_base();
        i.us_tb_status = UsTradeOrBusinessStatus::NoUsTradeOrBusiness;
        i.limited_agent_authorization_in_place = false;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("§ 6038C(c)")));
    }

    #[test]
    fn section_6501_c8_sol_tolled_on_non_filing() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        let r = check(&i);
        assert!(r.section_6501_c8_sol_tolled);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6501(c)(8)") && f.contains("OPEN INDEFINITELY")));
    }

    #[test]
    fn section_6501_c8_sol_not_tolled_when_filed() {
        let r = check(&engaged_base());
        assert!(!r.section_6501_c8_sol_tolled);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&engaged_base());
        assert!(r.citation.contains("§ 6038C(a)-(d)"));
        assert!(r
            .citation
            .contains("Omnibus Budget Reconciliation Act of 1990 § 11315"));
        assert!(r.citation.contains("Pub. L. 101-508"));
        assert!(r.citation.contains("November 5, 1990"));
        assert!(r.citation.contains("§ 6038A(b)-(e)"));
        assert!(r.citation.contains("§ 6501(c)(8)"));
        assert!(r.citation.contains("§ 864(b) and § 864(b)(2)"));
        assert!(r.citation.contains("§ 882"));
        assert!(r.citation.contains("§ 7602 and § 7603 and § 7604"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.6038A-1 through § 1.6038A-7"));
        assert!(r.citation.contains("Form 5472"));
        assert!(r.citation.contains("IRM 8.11.5"));
        assert!(r.citation.contains("IRM 20.1.9"));
    }

    #[test]
    fn note_pins_subsection_a_engaged_at_any_time() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038C(a)")
            && n.contains("ANY TIME during a taxable year")
            && n.contains("§ 6038A(b)")));
    }

    #[test]
    fn note_pins_subsection_b_cross_reference_to_6038a_penalties() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038C(b)")
            && n.contains("$25,000 base")
            && n.contains("$25,000/30-day continuation")
            && n.contains("UNCAPPED after 90-day")));
    }

    #[test]
    fn note_pins_subsection_c_limited_agent_authorization() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038C(c)")
            && n.contains("LIMITED AGENT")
            && n.contains("§ 7602")
            && n.contains("§ 7603")
            && n.contains("§ 7604")));
    }

    #[test]
    fn note_pins_subsection_d_cross_reference_definitions() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038C(d)")
            && n.contains("§ 6038A(c)")
            && n.contains("direct cross-reference")));
    }

    #[test]
    fn note_pins_864b2_trading_safe_harbor() {
        let r = check(&engaged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 864(b)(2) trading safe harbor")
                && n.contains("foreign person NOT a dealer")
                && n.contains("trades for own account")));
    }

    #[test]
    fn note_pins_882_eci_taxation() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 882")
            && n.contains("effectively connected")
            && n.contains("§ 884")));
    }

    #[test]
    fn note_pins_omnibus_budget_reconciliation_1990_origin() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(|n| n
            .contains("Omnibus Budget Reconciliation Act of 1990 § 11315")
            && n.contains("Pub. L. 101-508")
            && n.contains("November 5, 1990")
            && n.contains("March 20, 1990")));
    }

    #[test]
    fn note_pins_6501_c8_indefinite_sol() {
        let r = check(&engaged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6501(c)(8)") && n.contains("OPEN INDEFINITELY")));
    }

    #[test]
    fn note_pins_anti_avoidance_backstop_role() {
        let r = check(&engaged_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6038C is the anti-avoidance backstop")
                && n.contains("§ 6038A")
                && n.contains("§ 6038B")));
    }

    #[test]
    fn note_pins_form_5472_shared_with_6038a() {
        let r = check(&engaged_base());
        assert!(r.notes.iter().any(
            |n| n.contains("Treas. Reg. § 1.6038A-1 through § 1.6038A-7")
                && n.contains("Form 5472 instructions")
        ));
    }

    #[test]
    fn us_tb_status_truth_table_three_cells() {
        for (status, exp_subject) in [
            (UsTradeOrBusinessStatus::EngagedInUsTradeOrBusiness, true),
            (
                UsTradeOrBusinessStatus::Section864b2TradingSafeHarbor,
                false,
            ),
            (UsTradeOrBusinessStatus::NoUsTradeOrBusiness, false),
        ] {
            let mut i = engaged_base();
            i.us_tb_status = status;
            let r = check(&i);
            assert_eq!(
                r.subject_to_section_6038c, exp_subject,
                "status={:?}",
                status
            );
        }
    }

    #[test]
    fn safe_harbor_uniquely_engages_864b2_flag_invariant() {
        let mut sh = engaged_base();
        sh.us_tb_status = UsTradeOrBusinessStatus::Section864b2TradingSafeHarbor;
        let r_sh = check(&sh);
        assert!(r_sh.section_864b2_safe_harbor_engaged);

        for status in [
            UsTradeOrBusinessStatus::EngagedInUsTradeOrBusiness,
            UsTradeOrBusinessStatus::NoUsTradeOrBusiness,
        ] {
            let mut i = engaged_base();
            i.us_tb_status = status;
            let r = check(&i);
            assert!(!r.section_864b2_safe_harbor_engaged, "status={:?}", status);
        }
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = u32::MAX;
        let r = check(&i);
        let _ = r.total_penalty_cents;
        assert_eq!(r.base_penalty_cents, 2_500_000);
    }

    #[test]
    fn multiple_failure_reasons_stack() {
        let mut i = engaged_base();
        i.form_5472_filed = false;
        i.records_maintained = false;
        i.limited_agent_authorization_in_place = false;
        i.days_since_irs_notification = 200;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
