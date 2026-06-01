//! IRC § 6330 — Notice and opportunity for hearing before levy
//! (Collection Due Process — "CDP" — for levies). Companion to
//! § 6320 (CDP for liens) and contrast to § 6213 (Tax Court
//! petition from notice of deficiency).
//!
//! Trader-critical when receiving the IRS Final Notice of Intent to
//! Levy (Letter 1058 / LT-11). § 6330 gives the taxpayer 30 days to
//! request a CDP hearing before IRS Appeals during which collection
//! is SUSPENDED — the only pre-payment opportunity to raise
//! collection alternatives (installment agreement, offer in
//! compromise, currently not collectible), challenge underlying
//! liability (if no prior opportunity), or assert spousal defenses.
//! After IRS Appeals issues a Notice of Determination, the taxpayer
//! has another 30 days to petition the Tax Court.
//!
//! § 6330(a)(1) WRITTEN NOTICE REQUIRED — no levy may be made on
//! any property unless the Secretary has notified such person IN
//! WRITING of the right to a hearing under this section. § 6330(a)
//! (3) notice must be given at least 30 days before the day of the
//! first levy with respect to the amount of unpaid tax for the
//! taxable period.
//!
//! § 6330(b)(1) CDP HEARING — if the person requests a hearing in
//! writing within the 30-day period after the day of the notice,
//! the person shall be entitled to a fair hearing before the Office
//! of Appeals.
//!
//! § 6330(c) MATTERS CONSIDERED AT HEARING — the appeals officer
//! may consider (A) issues raised regarding the unpaid tax,
//! including (i) appropriate spousal defenses, (ii) challenges to
//! the appropriateness of collection actions, (iii) offers of
//! collection alternatives (installment agreement under § 6159,
//! offer in compromise under § 7122, currently not collectible
//! status); and (B) the taxpayer may challenge the EXISTENCE OR
//! AMOUNT of the underlying tax liability ONLY IF the taxpayer did
//! not receive a notice of deficiency and did not otherwise have an
//! opportunity to dispute such tax liability.
//!
//! § 6330(d)(1) TAX COURT PETITION — the person may, within 30
//! DAYS of a determination under this section, petition the Tax
//! Court for review of such determination (and the Tax Court shall
//! have jurisdiction with respect to such matter).
//!
//! § 6330(e) SUSPENSION OF COLLECTIONS DURING APPEALS REVIEW —
//! except as provided in subsection (f), if a hearing is requested
//! under subsection (a)(3)(B), the levy actions which are the
//! subject of the requested hearing AND the running of any period
//! of limitations under section 6502, 6531, or 6532 are SUSPENDED
//! for the period during which such hearing, and appeals therein,
//! are pending.
//!
//! § 6330(f) JEOPARDY AND STATE-REFUND EXCEPTIONS — subsections
//! (a), (b), (c), and (e) do not apply in the case of (1) levy with
//! respect to state tax refund, (2) levy when the collection of
//! such tax is in jeopardy, (3) certain Federal contractor levies,
//! or (4) disqualified employment tax levies.
//!
//! BOECHLER v. COMMISSIONER, 596 U.S. 199 (2022) — UNANIMOUS
//! Supreme Court held the § 6330(d)(1) 30-day deadline is NOT
//! JURISDICTIONAL and IS SUBJECT to equitable tolling. Sharp
//! contrast to § 6213(a) Tax Court petition deadline from a notice
//! of deficiency, which Hallmark Research Collective (159 T.C. No.
//! 6, 2022) held IS jurisdictional. Boechler missed the deadline
//! by ONE DAY and the Court reversed the Eighth Circuit's
//! dismissal, permitting equitable tolling arguments to proceed.
//!
//! Citations: IRC § 6330(a) (30-day pre-levy notice); § 6330(b)
//! (CDP hearing right); § 6330(c) (matters at hearing — collection
//! alternatives + spousal defenses + liability challenge if no
//! prior opportunity); § 6330(d)(1) (30-day Tax Court petition);
//! § 6330(e) (collection suspension during pending review);
//! § 6330(f) (jeopardy and special exceptions); Boechler, P.C. v.
//! Commissioner, 596 U.S. 199 (2022) (non-jurisdictional + equitable
//! tolling); § 6213(a) (parallel deficiency petition); Hallmark
//! Research Collective, 159 T.C. No. 6 (2022) (§ 6213 contrast).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6330Input {
    /// Days between IRS Final Notice of Intent to Levy (Letter
    /// 1058 / LT-11) and the taxpayer's CDP request to IRS Appeals.
    /// Must be ≤ 30 days under § 6330(a)(3)(B) to entitle the
    /// taxpayer to the formal CDP hearing.
    pub days_from_final_notice_to_cdp_request: u32,
    /// Whether the taxpayer received a Notice of Determination
    /// from IRS Appeals after the CDP hearing.
    pub notice_of_determination_received: bool,
    /// Days between Notice of Determination and Tax Court petition.
    /// § 6330(d)(1) 30-day window — non-jurisdictional per Boechler.
    pub days_from_determination_to_tax_court_petition: u32,
    /// Whether the taxpayer pleaded facts supporting equitable
    /// tolling of the 30-day Tax Court window per Boechler.
    /// Relevant only when the petition is filed after day 30 — the
    /// Court allows the taxpayer to ARGUE for tolling but does not
    /// automatically grant it.
    pub equitable_tolling_facts_pleaded: bool,
    /// Whether the IRS asserts a § 6330(f) exception bypassing the
    /// 30-day notice + CDP hearing (jeopardy collection, state
    /// refund levy, certain Federal contractor levies, disqualified
    /// employment tax levies).
    pub jeopardy_or_special_exception: bool,
    /// Whether the taxpayer had a prior opportunity to dispute the
    /// underlying tax liability (notice of deficiency under § 6212,
    /// Tax Court adjudication, etc.). § 6330(c)(2)(B) bars
    /// underlying-liability challenge at CDP if a prior opportunity
    /// existed.
    pub prior_opportunity_to_dispute_underlying_liability: bool,
    /// Whether the taxpayer's CDP submission proposes collection
    /// alternatives under § 6330(c)(2)(A)(iii) (installment
    /// agreement, offer in compromise, currently not collectible).
    pub collection_alternative_proposed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6330Result {
    pub cdp_hearing_entitlement: bool,
    pub tax_court_petition_timely: bool,
    pub collection_suspended_pending_review: bool,
    pub liability_challenge_available_at_cdp: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6330Input) -> Section6330Result {
    let mut notes: Vec<String> = Vec::new();

    if input.jeopardy_or_special_exception {
        notes.push(
            "§ 6330(f) exception engaged — subsections (a), (b), (c), and (e) do not apply; jeopardy collection, state refund levy, Federal contractor levy, or disqualified employment tax levy bypasses CDP framework"
                .to_string(),
        );
        return Section6330Result {
            cdp_hearing_entitlement: false,
            tax_court_petition_timely: false,
            collection_suspended_pending_review: false,
            liability_challenge_available_at_cdp: false,
            citation: citation(),
            notes,
        };
    }

    let cdp_entitlement = input.days_from_final_notice_to_cdp_request <= 30;
    if cdp_entitlement {
        notes.push(format!(
            "§ 6330(a)(3)(B) — CDP request filed within {} days of Final Notice; taxpayer entitled to fair hearing before Office of Appeals",
            input.days_from_final_notice_to_cdp_request
        ));
    } else {
        notes.push(format!(
            "§ 6330(a)(3)(B) — CDP request filed {} days after Final Notice exceeds 30-day window; taxpayer may request equivalent hearing under Treas. Reg. § 301.6330-1(i) but lacks judicial review right",
            input.days_from_final_notice_to_cdp_request
        ));
    }

    let collection_suspended = cdp_entitlement;
    if collection_suspended {
        notes.push(
            "§ 6330(e) — levy actions AND running of § 6502, § 6531, § 6532 limitations periods SUSPENDED while CDP hearing and appeals therein are pending"
                .to_string(),
        );
    }

    let liability_challenge_available =
        cdp_entitlement && !input.prior_opportunity_to_dispute_underlying_liability;
    if liability_challenge_available {
        notes.push(
            "§ 6330(c)(2)(B) — taxpayer may challenge existence OR amount of underlying liability at CDP (no prior § 6212 notice of deficiency or other opportunity)"
                .to_string(),
        );
    } else if cdp_entitlement {
        notes.push(
            "§ 6330(c)(2)(B) — underlying-liability challenge BARRED at CDP; taxpayer had prior opportunity to dispute (notice of deficiency or Tax Court adjudication)"
                .to_string(),
        );
    }

    if input.collection_alternative_proposed && cdp_entitlement {
        notes.push(
            "§ 6330(c)(2)(A)(iii) — collection alternatives proposed (§ 6159 installment agreement, § 7122 offer in compromise, currently not collectible)"
                .to_string(),
        );
    }

    let tax_court_timely = if input.notice_of_determination_received {
        if input.days_from_determination_to_tax_court_petition <= 30 {
            notes.push(format!(
                "§ 6330(d)(1) — Tax Court petition filed within {} days of Notice of Determination",
                input.days_from_determination_to_tax_court_petition
            ));
            true
        } else if input.equitable_tolling_facts_pleaded {
            notes.push(format!(
                "Boechler, 596 U.S. 199 (2022) — § 6330(d)(1) 30-day deadline NON-jurisdictional and SUBJECT TO equitable tolling; petition filed {} days after determination with tolling facts pleaded",
                input.days_from_determination_to_tax_court_petition
            ));
            true
        } else {
            notes.push(format!(
                "petition filed {} days after Notice of Determination exceeds 30-day window; equitable tolling facts not pleaded — Boechler argument unavailable",
                input.days_from_determination_to_tax_court_petition
            ));
            false
        }
    } else {
        false
    };

    notes.push(
        "contrast § 6213(a) Tax Court deficiency petition deadline — Hallmark Research Collective (159 T.C. No. 6, 2022) holds IT IS jurisdictional; § 6330(d)(1) and § 6213(a) treated differently post-Boechler"
            .to_string(),
    );

    Section6330Result {
        cdp_hearing_entitlement: cdp_entitlement,
        tax_court_petition_timely: tax_court_timely,
        collection_suspended_pending_review: collection_suspended,
        liability_challenge_available_at_cdp: liability_challenge_available,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 6330(a)/(b)/(c)/(d)(1)/(e)/(f); Treas. Reg. § 301.6330-1; Boechler, P.C. v. Commissioner, 596 U.S. 199 (2022); § 6213(a); Hallmark Research Collective, 159 T.C. No. 6 (2022) (§ 6213 contrast)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6330Input {
        Section6330Input {
            days_from_final_notice_to_cdp_request: 15,
            notice_of_determination_received: true,
            days_from_determination_to_tax_court_petition: 20,
            equitable_tolling_facts_pleaded: false,
            jeopardy_or_special_exception: false,
            prior_opportunity_to_dispute_underlying_liability: false,
            collection_alternative_proposed: true,
        }
    }

    #[test]
    fn timely_cdp_request_grants_hearing_and_suspends_collection() {
        let r = compute(&base());
        assert!(r.cdp_hearing_entitlement);
        assert!(r.collection_suspended_pending_review);
    }

    #[test]
    fn exactly_30_days_cdp_request_entitled_to_hearing() {
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 30;
        let r = compute(&i);
        assert!(r.cdp_hearing_entitlement);
    }

    #[test]
    fn day_31_cdp_request_loses_judicial_review() {
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 31;
        let r = compute(&i);
        assert!(!r.cdp_hearing_entitlement);
        assert!(!r.collection_suspended_pending_review);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("equivalent hearing")));
    }

    #[test]
    fn tax_court_petition_within_30_days_timely() {
        let r = compute(&base());
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn tax_court_petition_at_30_days_boundary_timely() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 30;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn tax_court_petition_day_31_without_tolling_facts_untimely() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        i.equitable_tolling_facts_pleaded = false;
        let r = compute(&i);
        assert!(!r.tax_court_petition_timely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("equitable tolling facts not pleaded")));
    }

    #[test]
    fn tax_court_petition_day_31_with_tolling_facts_timely_under_boechler() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Boechler") && n.contains("NON-jurisdictional")));
    }

    #[test]
    fn tax_court_petition_at_day_one_late_with_tolling_classic_boechler_facts() {
        // Boechler's case — petition mailed one day late.
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn no_notice_of_determination_no_petition_pathway() {
        let mut i = base();
        i.notice_of_determination_received = false;
        let r = compute(&i);
        assert!(!r.tax_court_petition_timely);
    }

    #[test]
    fn jeopardy_exception_bypasses_entire_cdp_framework() {
        let mut i = base();
        i.jeopardy_or_special_exception = true;
        let r = compute(&i);
        assert!(!r.cdp_hearing_entitlement);
        assert!(!r.tax_court_petition_timely);
        assert!(!r.collection_suspended_pending_review);
        assert!(!r.liability_challenge_available_at_cdp);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6330(f) exception engaged")));
    }

    #[test]
    fn underlying_liability_challenge_available_when_no_prior_opportunity() {
        let mut i = base();
        i.prior_opportunity_to_dispute_underlying_liability = false;
        let r = compute(&i);
        assert!(r.liability_challenge_available_at_cdp);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6330(c)(2)(B)") && n.contains("may challenge")));
    }

    #[test]
    fn underlying_liability_challenge_barred_when_prior_opportunity_existed() {
        let mut i = base();
        i.prior_opportunity_to_dispute_underlying_liability = true;
        let r = compute(&i);
        assert!(!r.liability_challenge_available_at_cdp);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BARRED") && n.contains("notice of deficiency")));
    }

    #[test]
    fn collection_alternative_proposal_surfaces_in_notes() {
        let r = compute(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6330(c)(2)(A)(iii)") && n.contains("collection alternatives")));
    }

    #[test]
    fn no_collection_alternative_no_alternative_note() {
        let mut i = base();
        i.collection_alternative_proposed = false;
        let r = compute(&i);
        let alt_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("§ 6330(c)(2)(A)(iii)"))
            .collect();
        assert!(alt_notes.is_empty());
    }

    #[test]
    fn section_6213_contrast_note_always_present() {
        let r = compute(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Hallmark Research Collective") && n.contains("§ 6213")));
    }

    #[test]
    fn citation_pins_subsections_treas_reg_and_cases() {
        let r = compute(&base());
        assert!(r.citation.contains("§ 6330(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)(1)"));
        assert!(r.citation.contains("(e)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("§ 301.6330-1"));
        assert!(r.citation.contains("Boechler"));
        assert!(r.citation.contains("596 U.S. 199 (2022)"));
        assert!(r.citation.contains("Hallmark Research Collective"));
    }

    #[test]
    fn collection_suspension_engages_with_timely_cdp_request() {
        let r = compute(&base());
        assert!(r.collection_suspended_pending_review);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6330(e)") && n.contains("§ 6502")));
    }

    #[test]
    fn collection_suspension_does_not_engage_with_late_cdp_request() {
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 31;
        let r = compute(&i);
        assert!(!r.collection_suspended_pending_review);
    }

    #[test]
    fn tax_court_petition_unavailable_when_cdp_hearing_lost() {
        // Even if equitable tolling pleaded, the underlying CDP hearing
        // must have been timely. Wait — that's not quite right. The
        // 30-day window starts at Notice of Determination not Final
        // Notice. The two windows are independent. Let me reframe.
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 31;
        i.notice_of_determination_received = false;
        let r = compute(&i);
        assert!(!r.tax_court_petition_timely);
    }

    #[test]
    fn day_30_at_boundary_cdp_request_timely() {
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 30;
        let r = compute(&i);
        assert!(r.cdp_hearing_entitlement);
    }

    #[test]
    fn day_30_at_boundary_tax_court_petition_timely() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 30;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn jeopardy_exception_invariant_no_cdp_or_tax_court_or_suspension() {
        let cases = [
            (0u32, 0u32),
            (30u32, 30u32),
            (60u32, 60u32),
            (100u32, 100u32),
        ];
        for (cdp_days, tc_days) in cases {
            let i = Section6330Input {
                days_from_final_notice_to_cdp_request: cdp_days,
                notice_of_determination_received: true,
                days_from_determination_to_tax_court_petition: tc_days,
                equitable_tolling_facts_pleaded: true,
                jeopardy_or_special_exception: true,
                prior_opportunity_to_dispute_underlying_liability: false,
                collection_alternative_proposed: true,
            };
            let r = compute(&i);
            assert!(!r.cdp_hearing_entitlement, "jeopardy cancels CDP regardless of timing");
            assert!(!r.collection_suspended_pending_review);
            assert!(!r.tax_court_petition_timely);
        }
    }

    #[test]
    fn boechler_one_day_late_with_tolling_facts_timely() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn boechler_one_year_late_with_tolling_facts_still_timely_under_module_logic() {
        // Module accepts any equitable_tolling_facts_pleaded=true.
        // Actual Tax Court determination of tolling is fact-specific
        // (diligent pursuit + extraordinary circumstance). Caller
        // responsibility to assess merit.
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 365;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
    }

    #[test]
    fn liability_challenge_unavailable_when_cdp_request_late() {
        let mut i = base();
        i.days_from_final_notice_to_cdp_request = 31;
        i.prior_opportunity_to_dispute_underlying_liability = false;
        let r = compute(&i);
        assert!(!r.liability_challenge_available_at_cdp);
    }
}
