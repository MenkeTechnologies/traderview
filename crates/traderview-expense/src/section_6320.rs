//! IRC § 6320 — Notice and opportunity for hearing upon filing of
//! notice of lien (Collection Due Process — "CDP" — for liens).
//! Parallel framework to § 6330 (CDP for levies, built iter 280)
//! with lien-specific mechanics.
//!
//! Trader-critical when the IRS files a Notice of Federal Tax Lien
//! (NFTL) — a public record asserting the government's claim against
//! all of the taxpayer's property. § 6320 gives the taxpayer the
//! right to a CDP hearing before IRS Appeals to challenge the lien
//! filing, propose collection alternatives, or raise spousal
//! defenses, with subsequent right to Tax Court review.
//!
//! § 6320(a)(1) WRITTEN NOTICE REQUIRED — Secretary shall notify in
//! writing the person described in § 6321 of the filing of a notice
//! of lien under § 6323. § 6320(a)(2) — such notice shall be (A)
//! given in person, left at the dwelling or usual place of business,
//! or sent by certified or registered mail to such person's last
//! known address, and (B) given NOT MORE THAN 5 BUSINESS DAYS after
//! the day of the filing of the notice of lien.
//!
//! § 6320(a)(3) INFORMATION INCLUDED IN NOTICE — notice (Letter
//! 3172) shall include (A) amount of unpaid tax, (B) right to
//! request CDP hearing within 30-day period beginning DAY AFTER the
//! end of the 5-business-day notice period, (C) administrative
//! appeals available and procedures, (D) provisions of § 6321 and
//! § 6322 relating to liens and their continuance.
//!
//! § 6320(b)(1) RIGHT TO FAIR HEARING — if the person requests a
//! hearing in writing within the 30-day period described in
//! subsection (a)(3)(B), such person shall be entitled to a fair
//! hearing.
//!
//! § 6320(c) ISSUES CONSIDERED — for purposes of this section,
//! subsections (c), (d) (other than paragraph (2)(B) thereof), and
//! (e) of § 6330 shall apply. § 6330(c)(2)(A) — collection
//! alternatives, spousal defenses, appropriateness of collection.
//! § 6330(c)(2)(B) — underlying-liability challenge IF no prior
//! opportunity. Lien-specific issues: lien WITHDRAWAL under § 6323(j),
//! SUBORDINATION under § 6325(d), and DISCHARGE under § 6325(b).
//!
//! § 6320(d) PROCEEDINGS GOVERNED BY § 6330 — proceedings governed
//! by § 6330(d) — 30-day Tax Court petition window after Notice of
//! Determination from IRS Appeals.
//!
//! KEY DIFFERENCES FROM § 6330 (LEVY):
//!   - Lien NFTL notice deadline = 5 BUSINESS DAYS after filing
//!     (§ 6320(a)(2)(B)) — § 6330 pre-levy notice = at least 30 days
//!     before levy (§ 6330(a)(3)(B))
//!   - CDP request window = 30 days starting DAY AFTER end of 5-
//!     business-day period (§ 6320(a)(3)(B))
//!   - LIEN REMAINS IN PLACE during CDP hearing (no automatic
//!     withdrawal) — § 6330(e) levy-suspension does NOT apply to
//!     lien-filing context
//!   - Underlying tax liability stays in force; lien public record
//!     persists pending resolution
//!
//! BOECHLER v. COMMISSIONER, 596 U.S. 199 (2022) — held § 6330(d)(1)
//! 30-day Tax Court petition deadline NON-jurisdictional + equitable
//! tolling. § 6320(d) incorporates § 6330(d) by reference, so
//! Boechler's holding likely extends to lien appeals.
//!
//! Citations: IRC § 6320(a)(1)/(a)(2)/(a)(3) (notice content + 5-
//! business-day timeline); § 6320(b)(1) (CDP hearing right); § 6320(c)
//! (incorporates § 6330(c) issues + lien-specific § 6323(j) withdrawal /
//! § 6325(d) subordination / § 6325(b) discharge); § 6320(d)
//! (incorporates § 6330(d) Tax Court framework); 26 CFR § 301.6320-1
//! (regulations); Boechler, P.C. v. Commissioner, 596 U.S. 199
//! (2022) (non-jurisdictional + equitable tolling).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6320Input {
    /// Business days between IRS filing of NFTL and mailing of CDP
    /// notice (Letter 3172). § 6320(a)(2)(B) requires ≤ 5 business
    /// days. Failure does not invalidate the lien but exposes the
    /// IRS to a procedural defect challenge at the CDP hearing.
    pub business_days_from_nftl_filing_to_notice: u32,
    /// Days between receipt of CDP notice and taxpayer's request
    /// for CDP hearing. § 6320(a)(3)(B) — 30-day window beginning
    /// day AFTER end of 5-business-day notice period. The 30-day
    /// clock effectively starts ~5 business days after NFTL filing.
    pub days_from_notice_to_cdp_request: u32,
    /// Whether the taxpayer received a Notice of Determination from
    /// IRS Appeals after the CDP hearing.
    pub notice_of_determination_received: bool,
    /// Days between Notice of Determination and Tax Court petition.
    /// § 6320(d) incorporates § 6330(d)(1) 30-day window — Boechler
    /// non-jurisdictional + equitable tolling likely extends here.
    pub days_from_determination_to_tax_court_petition: u32,
    /// Whether the taxpayer pleaded facts supporting equitable
    /// tolling of the 30-day Tax Court window per Boechler.
    pub equitable_tolling_facts_pleaded: bool,
    /// Whether the taxpayer had a prior opportunity to dispute the
    /// underlying tax liability. § 6330(c)(2)(B) — incorporated by
    /// § 6320(c) — bars underlying-liability challenge if prior
    /// opportunity existed.
    pub prior_opportunity_to_dispute_underlying_liability: bool,
    /// Whether the taxpayer requests collection alternatives or
    /// lien-specific relief (§ 6323(j) withdrawal, § 6325(d)
    /// subordination, § 6325(b) discharge, installment agreement,
    /// offer in compromise).
    pub collection_alternative_or_lien_relief_requested: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6320Result {
    pub irs_notice_compliant: bool,
    pub cdp_hearing_entitlement: bool,
    pub tax_court_petition_timely: bool,
    pub lien_remains_in_place_during_review: bool,
    pub liability_challenge_available_at_cdp: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6320Input) -> Section6320Result {
    let mut notes: Vec<String> = Vec::new();

    let irs_notice_compliant = input.business_days_from_nftl_filing_to_notice <= 5;
    if irs_notice_compliant {
        notes.push(format!(
            "§ 6320(a)(2)(B) — IRS mailed CDP notice within {} business days of NFTL filing; ≤ 5-business-day deadline satisfied",
            input.business_days_from_nftl_filing_to_notice
        ));
    } else {
        notes.push(format!(
            "§ 6320(a)(2)(B) procedural defect — IRS mailed CDP notice {} business days after NFTL filing; exceeds 5-business-day deadline; taxpayer may raise at CDP hearing (but lien itself remains valid)",
            input.business_days_from_nftl_filing_to_notice
        ));
    }

    let cdp_entitlement = input.days_from_notice_to_cdp_request <= 30;
    if cdp_entitlement {
        notes.push(format!(
            "§ 6320(a)(3)(B) — CDP request filed within {} days; 30-day window starts day AFTER 5-business-day notice period",
            input.days_from_notice_to_cdp_request
        ));
    } else {
        notes.push(format!(
            "§ 6320(a)(3)(B) — CDP request {} days after notice exceeds 30-day window; taxpayer may request equivalent hearing under Treas. Reg. § 301.6320-1(i) but lacks judicial review right",
            input.days_from_notice_to_cdp_request
        ));
    }

    notes.push(
        "lien remains in place during CDP hearing — unlike § 6330 levy framework, NFTL is NOT automatically withdrawn pending review"
            .to_string(),
    );

    let liability_challenge_available =
        cdp_entitlement && !input.prior_opportunity_to_dispute_underlying_liability;
    if liability_challenge_available {
        notes.push(
            "§ 6320(c) (incorporating § 6330(c)(2)(B)) — taxpayer may challenge underlying tax liability at CDP (no prior opportunity)"
                .to_string(),
        );
    } else if cdp_entitlement {
        notes.push(
            "§ 6320(c) (incorporating § 6330(c)(2)(B)) — underlying-liability challenge BARRED; taxpayer had prior opportunity (§ 6212 notice of deficiency or Tax Court adjudication)"
                .to_string(),
        );
    }

    if input.collection_alternative_or_lien_relief_requested && cdp_entitlement {
        notes.push(
            "§ 6320(c) (incorporating § 6330(c)(2)(A)) — collection alternatives + lien-specific relief available: § 6323(j) lien WITHDRAWAL, § 6325(d) SUBORDINATION, § 6325(b) DISCHARGE, § 6159 installment agreement, § 7122 offer in compromise"
                .to_string(),
        );
    }

    let tax_court_timely = if input.notice_of_determination_received {
        if input.days_from_determination_to_tax_court_petition <= 30 {
            notes.push(format!(
                "§ 6320(d) (incorporating § 6330(d)(1)) — Tax Court petition filed within {} days of Notice of Determination",
                input.days_from_determination_to_tax_court_petition
            ));
            true
        } else if input.equitable_tolling_facts_pleaded {
            notes.push(format!(
                "Boechler, 596 U.S. 199 (2022) (likely extends via § 6320(d) incorporation) — petition filed {} days after determination with equitable tolling facts pleaded; non-jurisdictional deadline subject to tolling argument",
                input.days_from_determination_to_tax_court_petition
            ));
            true
        } else {
            notes.push(format!(
                "petition filed {} days after Notice of Determination exceeds 30-day window; equitable tolling facts not pleaded",
                input.days_from_determination_to_tax_court_petition
            ));
            false
        }
    } else {
        false
    };

    Section6320Result {
        irs_notice_compliant,
        cdp_hearing_entitlement: cdp_entitlement,
        tax_court_petition_timely: tax_court_timely,
        lien_remains_in_place_during_review: true,
        liability_challenge_available_at_cdp: liability_challenge_available,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 6320(a)(1)/(a)(2)/(a)(3)/(b)(1)/(c)/(d); 26 CFR § 301.6320-1; § 6330(c)(2)(A)/(B) incorporated; § 6323(j)/(b)/(c); § 6325(b)/(d); Boechler, P.C. v. Commissioner, 596 U.S. 199 (2022)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6320Input {
        Section6320Input {
            business_days_from_nftl_filing_to_notice: 3,
            days_from_notice_to_cdp_request: 15,
            notice_of_determination_received: true,
            days_from_determination_to_tax_court_petition: 20,
            equitable_tolling_facts_pleaded: false,
            prior_opportunity_to_dispute_underlying_liability: false,
            collection_alternative_or_lien_relief_requested: true,
        }
    }

    #[test]
    fn timely_notice_and_cdp_request_grant_hearing() {
        let r = compute(&base());
        assert!(r.irs_notice_compliant);
        assert!(r.cdp_hearing_entitlement);
    }

    #[test]
    fn exactly_5_business_days_notice_compliant() {
        let mut i = base();
        i.business_days_from_nftl_filing_to_notice = 5;
        let r = compute(&i);
        assert!(r.irs_notice_compliant);
    }

    #[test]
    fn business_day_6_notice_procedural_defect() {
        let mut i = base();
        i.business_days_from_nftl_filing_to_notice = 6;
        let r = compute(&i);
        assert!(!r.irs_notice_compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("procedural defect")));
    }

    #[test]
    fn cdp_request_at_30_days_boundary_timely() {
        let mut i = base();
        i.days_from_notice_to_cdp_request = 30;
        let r = compute(&i);
        assert!(r.cdp_hearing_entitlement);
    }

    #[test]
    fn cdp_request_day_31_loses_judicial_review() {
        let mut i = base();
        i.days_from_notice_to_cdp_request = 31;
        let r = compute(&i);
        assert!(!r.cdp_hearing_entitlement);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("equivalent hearing")));
    }

    #[test]
    fn lien_remains_in_place_during_review_invariant() {
        let r = compute(&base());
        assert!(r.lien_remains_in_place_during_review);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("NOT automatically withdrawn")));
    }

    #[test]
    fn collection_alternatives_and_lien_relief_note_when_requested() {
        let r = compute(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6323(j) lien WITHDRAWAL") && n.contains("§ 6325(d) SUBORDINATION") && n.contains("§ 6325(b) DISCHARGE")));
    }

    #[test]
    fn no_collection_alternatives_no_relief_note() {
        let mut i = base();
        i.collection_alternative_or_lien_relief_requested = false;
        let r = compute(&i);
        let relief_notes: Vec<_> = r
            .notes
            .iter()
            .filter(|n| n.contains("§ 6323(j) lien WITHDRAWAL"))
            .collect();
        assert!(relief_notes.is_empty());
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
    fn tax_court_day_31_without_tolling_untimely() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        let r = compute(&i);
        assert!(!r.tax_court_petition_timely);
    }

    #[test]
    fn tax_court_day_31_with_tolling_timely_under_boechler_extension() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 31;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r.tax_court_petition_timely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Boechler") && n.contains("§ 6320(d) incorporation")));
    }

    #[test]
    fn underlying_liability_challenge_available_without_prior_opportunity() {
        let r = compute(&base());
        assert!(r.liability_challenge_available_at_cdp);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6320(c)") && n.contains("§ 6330(c)(2)(B)") && n.contains("may challenge")));
    }

    #[test]
    fn underlying_liability_challenge_barred_with_prior_opportunity() {
        let mut i = base();
        i.prior_opportunity_to_dispute_underlying_liability = true;
        let r = compute(&i);
        assert!(!r.liability_challenge_available_at_cdp);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BARRED") && n.contains("§ 6212")));
    }

    #[test]
    fn no_notice_of_determination_no_petition_pathway() {
        let mut i = base();
        i.notice_of_determination_received = false;
        let r = compute(&i);
        assert!(!r.tax_court_petition_timely);
    }

    #[test]
    fn citation_pins_subsections_and_incorporated_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§ 6320(a)(1)"));
        assert!(r.citation.contains("(a)(2)"));
        assert!(r.citation.contains("(a)(3)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("§ 301.6320-1"));
        assert!(r.citation.contains("§ 6330(c)(2)(A)/(B) incorporated"));
        assert!(r.citation.contains("§ 6323(j)"));
        assert!(r.citation.contains("§ 6325(b)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("Boechler"));
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
    fn lien_specific_relief_includes_withdrawal_subordination_discharge() {
        let r = compute(&base());
        let alts_note = r
            .notes
            .iter()
            .find(|n| n.contains("§ 6323(j)"))
            .expect("relief note expected");
        assert!(alts_note.contains("§ 6323(j)"));
        assert!(alts_note.contains("§ 6325(d)"));
        assert!(alts_note.contains("§ 6325(b)"));
        assert!(alts_note.contains("§ 6159"));
        assert!(alts_note.contains("§ 7122"));
    }

    #[test]
    fn five_business_day_window_distinct_from_section_6330_thirty_day() {
        let r = compute(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("5-business-day")));
    }

    #[test]
    fn liability_challenge_unavailable_when_cdp_request_late() {
        let mut i = base();
        i.days_from_notice_to_cdp_request = 31;
        i.prior_opportunity_to_dispute_underlying_liability = false;
        let r = compute(&i);
        assert!(!r.liability_challenge_available_at_cdp);
    }

    #[test]
    fn procedural_defect_does_not_invalidate_lien_itself() {
        let mut i = base();
        i.business_days_from_nftl_filing_to_notice = 30;
        let r = compute(&i);
        assert!(!r.irs_notice_compliant);
        assert!(r.lien_remains_in_place_during_review, "procedural defect does not lift lien");
    }

    #[test]
    fn zero_days_notice_obviously_compliant() {
        let mut i = base();
        i.business_days_from_nftl_filing_to_notice = 0;
        let r = compute(&i);
        assert!(r.irs_notice_compliant);
    }

    #[test]
    fn boechler_facts_extension_note_includes_section_6320d_incorporation() {
        let mut i = base();
        i.days_from_determination_to_tax_court_petition = 35;
        i.equitable_tolling_facts_pleaded = true;
        let r = compute(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6320(d) incorporation")));
    }

    #[test]
    fn cdp_request_at_business_day_5_combined_with_30_day_request_timely() {
        let mut i = base();
        i.business_days_from_nftl_filing_to_notice = 5;
        i.days_from_notice_to_cdp_request = 30;
        let r = compute(&i);
        assert!(r.irs_notice_compliant);
        assert!(r.cdp_hearing_entitlement);
    }

    #[test]
    fn lien_in_place_invariant_across_all_paths() {
        let cases = [
            (0u32, 0u32),
            (5u32, 30u32),
            (6u32, 31u32),
            (100u32, 100u32),
        ];
        for (bd, req_days) in cases {
            let mut i = base();
            i.business_days_from_nftl_filing_to_notice = bd;
            i.days_from_notice_to_cdp_request = req_days;
            let r = compute(&i);
            assert!(r.lien_remains_in_place_during_review, "lien always remains in place regardless of timing");
        }
    }
}
