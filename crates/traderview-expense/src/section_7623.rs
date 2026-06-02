//! IRC § 7623 — Expenses of detection of underpayments and
//! fraud, etc. — IRS Whistleblower Awards. Originally
//! enacted 1867 (discretionary awards). § 7623(b) MANDATORY
//! award program added by Tax Relief and Health Care Act of
//! 2006 § 406 (Pub. L. 109-432, December 20, 2006). § 7623(c)
//! "collected proceeds" definition added by Bipartisan
//! Budget Act of 2018 § 41108 (Pub. L. 115-123). § 7623(d)
//! anti-retaliation protections added by Taxpayer First Act
//! of 2019 § 1405(a) (Pub. L. 116-25, July 1, 2019).
//! Trader-critical: wealthy/sophisticated traders are
//! precisely the IRS Whistleblower Office target taxpayer
//! class — high gross income + complex tax positions makes
//! them disproportionately exposed to whistleblower tips
//! from disgruntled fund employees, ex-spouses, business
//! partners, or accountants. Companion to section_6663
//! (civil fraud penalty), section_7201 (tax evasion),
//! section_7202 (willful failure to collect), section_7206
//! (perjury/false statements), section_6701 (aiding/
//! abetting penalty), section_7430 (litigation costs),
//! section_6038d (FATCA Form 8938).
//!
//! **§ 7623(a) Discretionary awards (1867 original)** —
//! Secretary, under regulations prescribed by Secretary,
//! is authorized to pay sums necessary for: (1) detecting
//! underpayments of tax; and (2) detecting and bringing to
//! trial and punishment persons guilty of violating the
//! internal revenue laws or conniving at the same. Sums
//! paid out of proceeds of amounts (other than interest)
//! collected by reason of the information provided.
//!
//! **§ 7623(b)(1) Mandatory award framework** — if the
//! Secretary proceeds with any administrative or judicial
//! action described in subsection (a) based on information
//! brought to the Secretary's attention by an individual,
//! such individual shall receive as an award **at least
//! 15 percent but not more than 30 percent** of the
//! proceeds collected as a result of the action (including
//! any related actions) or from any settlement in response
//! to such action.
//!
//! **§ 7623(b)(2) Reduced or denied award** — in
//! determining amount of award:
//! 1. If action is based principally on disclosures of
//!    specific allegations (other than information provided
//!    by individual) resulting from a **judicial or
//!    administrative hearing**, government report,
//!    hearing, audit, or investigation, or from the news
//!    media — Whistleblower Office may award up to **10
//!    percent** of proceeds (taking into account the
//!    significance of the individual's information and
//!    role).
//! 2. If individual **planned and initiated** the actions
//!    that led to the underpayment of tax or violation —
//!    award may be appropriately reduced.
//!
//! **§ 7623(b)(3) Award denied if convicted** — if
//! individual is convicted of criminal conduct arising
//! from the role described in (b)(2) — **no award shall
//! be paid**.
//!
//! **§ 7623(b)(4) Tax Court appeal** — any determination
//! regarding an award under (b)(1), (2), or (3) may, within
//! **30 days** of such determination, be appealed to the
//! **Tax Court** (and Tax Court has jurisdiction).
//!
//! **§ 7623(b)(5) Application of this subsection** —
//! mandatory award provisions apply ONLY with respect to
//! any action against any taxpayer in which:
//! 1. The **tax, penalties, interest, additions to tax,
//!    and additional amounts in dispute exceed $2,000,000**;
//!    AND
//! 2. If the taxpayer is an individual, **gross income
//!    exceeds $200,000** for any taxable year subject to
//!    such action.
//!
//! **§ 7623(c) Collected proceeds (Bipartisan Budget Act
//! of 2018)** — for purposes of (a) and (b), proceeds
//! include:
//! 1. Penalties, interest, additions to tax, and
//!    additional amounts provided under internal revenue
//!    laws; AND
//! 2. Any proceeds arising from laws for which IRS is
//!    authorized to administer, enforce, or investigate,
//!    including **criminal fines and civil forfeitures**,
//!    and **violations of reporting requirements** (e.g.,
//!    FBAR penalties under 31 USC § 5321).
//!
//! **§ 7623(d) Anti-retaliation (Taxpayer First Act of
//! 2019 § 1405)** — no employer, or any officer, employee,
//! contractor, subcontractor, or agent of such employer
//! may discharge, demote, suspend, threaten, harass, or
//! discriminate against an employee in terms of
//! compensation, conditions, or privileges of employment
//! because of any lawful act done by the employee:
//! 1. To provide information regarding underpayment of
//!    tax OR conduct constituting a violation of internal
//!    revenue laws to (i) IRS; (ii) Treasury Inspector
//!    General for Tax Administration (TIGTA); (iii)
//!    Comptroller General; (iv) DOJ; (v) Congress; or (vi)
//!    person with supervisory authority over the employee.
//!
//! **Remedies for § 7623(d) retaliation**: reinstatement,
//! **double back pay with interest**, compensation for
//! special damages including litigation costs, expert
//! witness fees, and reasonable attorney fees.
//!
//! **IRS Whistleblower Office** — created by Tax Relief
//! and Health Care Act of 2006 § 406; reports to Treasury
//! Commissioner; processes Form 211 (Application for Award
//! for Original Information).
//!
//! **Award computation under Treas. Reg. § 301.7623-4** —
//! starts at 15% baseline; analyzed for increase to 22% or
//! 30% based on factors including: significance of
//! individual's information; extent of substantial
//! contribution; whether individual planned/initiated
//! noncompliance; etc.
//!
//! Citations: 26 USC § 7623(a)-(d); Tax Relief and Health
//! Care Act of 2006 § 406 (Pub. L. 109-432, December 20,
//! 2006); Bipartisan Budget Act of 2018 § 41108 (Pub. L.
//! 115-123); Taxpayer First Act of 2019 § 1405(a) (Pub. L.
//! 116-25, July 1, 2019); Treas. Reg. § 301.7623-1 to
//! § 301.7623-4; IRM 25.2.2 (Whistleblower Awards); Form
//! 211; 31 USC § 5321 (FBAR penalties).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AwardTier {
    /// § 7623(a) discretionary (pre-2006 original).
    DiscretionaryOnly,
    /// § 7623(b)(1) mandatory 15-30% — thresholds met,
    /// not based on public info, WB did not plan/initiate.
    MandatoryFullRange,
    /// § 7623(b)(2)(A) reduced to up to 10% — based on
    /// public information (news media, government report,
    /// audit, hearing).
    ReducedPublicInformation,
    /// § 7623(b)(2)(B) reduced — WB planned/initiated
    /// noncompliance.
    ReducedPlannedInitiated,
    /// § 7623(b)(3) denied — WB convicted of criminal
    /// conduct arising from role.
    DeniedConvicted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7623Input {
    /// Amount in dispute (tax + penalties + interest +
    /// additions to tax) in cents.
    pub amount_in_dispute_cents: u64,
    /// Taxpayer's gross income (for any taxable year at
    /// issue) in cents.
    pub taxpayer_gross_income_cents: u64,
    /// Whether the target taxpayer is an individual
    /// (otherwise corporation/entity — § 200K threshold
    /// inapplicable).
    pub taxpayer_is_individual: bool,
    /// Whether the action proceeded based on information
    /// brought by the whistleblower (gate for mandatory
    /// award).
    pub information_acted_on_by_irs: bool,
    /// Collected proceeds attributable to the action in
    /// cents (denominator for the award percentage).
    pub collected_proceeds_cents: u64,
    /// Whether collected proceeds include § 7623(c) items
    /// (criminal fines, civil forfeitures, FBAR penalties).
    pub proceeds_include_criminal_fines_civil_forfeitures: bool,
    /// Whether information was based principally on
    /// disclosures from public sources (news media,
    /// government report, audit, hearing) → § 7623(b)(2)(A).
    pub based_on_public_information: bool,
    /// Whether whistleblower planned/initiated the
    /// noncompliance → § 7623(b)(2)(B).
    pub whistleblower_planned_or_initiated: bool,
    /// Whether whistleblower was convicted of criminal
    /// conduct arising from role → § 7623(b)(3).
    pub whistleblower_convicted_of_role: bool,
    /// Award percentage (15-30) determined by IRS
    /// Whistleblower Office.
    pub award_percentage_bps: u32,
    /// Days between Whistleblower Office determination and
    /// petitioner's filing of Tax Court petition (must be
    /// ≤ 30 per § 7623(b)(4)).
    pub days_to_tax_court_appeal: u32,
    /// Whether employer retaliated against whistleblower
    /// after § 7623(d) protected disclosure.
    pub retaliation_occurred: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7623Result {
    pub tier: AwardTier,
    pub mandatory_thresholds_met: bool,
    pub two_million_threshold_met: bool,
    pub two_hundred_k_gross_income_threshold_met: bool,
    pub min_award_percentage_bps: u32,
    pub max_award_percentage_bps: u32,
    pub award_cents: u64,
    pub award_denied: bool,
    pub tax_court_appeal_timely: bool,
    pub anti_retaliation_violated: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7623Input) -> Section7623Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let two_million_met = input.amount_in_dispute_cents > 200_000_000;
    let two_hundred_k_met = if input.taxpayer_is_individual {
        input.taxpayer_gross_income_cents > 20_000_000
    } else {
        true
    };
    let mandatory_thresholds_met =
        two_million_met && two_hundred_k_met && input.information_acted_on_by_irs;

    let tier = if input.whistleblower_convicted_of_role {
        AwardTier::DeniedConvicted
    } else if input.whistleblower_planned_or_initiated {
        AwardTier::ReducedPlannedInitiated
    } else if input.based_on_public_information {
        AwardTier::ReducedPublicInformation
    } else if mandatory_thresholds_met {
        AwardTier::MandatoryFullRange
    } else {
        AwardTier::DiscretionaryOnly
    };

    let (min_bps, max_bps): (u32, u32) = match tier {
        AwardTier::MandatoryFullRange => (1500, 3000),
        AwardTier::ReducedPublicInformation => (0, 1000),
        AwardTier::ReducedPlannedInitiated => (0, 3000),
        AwardTier::DiscretionaryOnly => (0, 1500),
        AwardTier::DeniedConvicted => (0, 0),
    };

    let award_denied = matches!(tier, AwardTier::DeniedConvicted);
    let clamped_pct_bps = if award_denied {
        0
    } else {
        input.award_percentage_bps.clamp(min_bps, max_bps)
    };

    let award_cents = (input.collected_proceeds_cents.saturating_mul(clamped_pct_bps as u64))
        / 10_000;

    if matches!(tier, AwardTier::MandatoryFullRange)
        && (input.award_percentage_bps < 1500 || input.award_percentage_bps > 3000)
    {
        failure_reasons.push(format!(
            "26 USC § 7623(b)(1) — mandatory award must be between 15% and 30% (1500-3000 bps); IRS Whistleblower Office set {} bps which is outside mandatory band",
            input.award_percentage_bps
        ));
    }

    if matches!(tier, AwardTier::ReducedPublicInformation) && input.award_percentage_bps > 1000 {
        failure_reasons.push(format!(
            "26 USC § 7623(b)(2)(A) — public-information-based awards capped at 10% (1000 bps); requested {} bps exceeds cap",
            input.award_percentage_bps
        ));
    }

    let tax_court_appeal_timely = input.days_to_tax_court_appeal <= 30;
    if input.days_to_tax_court_appeal > 30 {
        failure_reasons.push(format!(
            "26 USC § 7623(b)(4) — Tax Court appeal must be filed within 30 days of award determination; filed at day {}",
            input.days_to_tax_court_appeal
        ));
    }

    let anti_retaliation_violated = input.retaliation_occurred;
    if anti_retaliation_violated {
        failure_reasons.push(
            "26 USC § 7623(d) (Taxpayer First Act of 2019 § 1405) — employer prohibited from discharging, demoting, suspending, threatening, harassing, or discriminating against whistleblower; remedies include reinstatement, DOUBLE back pay with interest, special damages, attorney fees".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 7623(a) (1867 original) — Secretary authorized to pay sums for (1) detecting underpayments of tax; and (2) detecting and bringing to trial and punishment persons guilty of violating internal revenue laws or conniving at the same; sums paid out of proceeds (other than interest) collected by reason of the information provided".to_string(),
        "26 USC § 7623(b)(1) — if IRS proceeds with administrative or judicial action based on whistleblower information, individual shall receive award of at least 15 percent but not more than 30 percent of collected proceeds".to_string(),
        "26 USC § 7623(b)(2)(A) — if action based principally on disclosures from judicial or administrative hearing, government report, hearing, audit, investigation, or news media — Whistleblower Office may award up to 10 percent".to_string(),
        "26 USC § 7623(b)(2)(B) — if whistleblower planned and initiated noncompliance — award may be appropriately reduced".to_string(),
        "26 USC § 7623(b)(3) — if whistleblower convicted of criminal conduct arising from role described in (b)(2) — no award shall be paid".to_string(),
        "26 USC § 7623(b)(4) — any award determination may, within 30 days, be appealed to Tax Court (Tax Court has jurisdiction)".to_string(),
        "26 USC § 7623(b)(5) — mandatory award applies ONLY where (A) amount in dispute exceeds $2,000,000 AND (B) if taxpayer is individual, gross income exceeds $200,000 for any taxable year at issue".to_string(),
        "26 USC § 7623(c) (Bipartisan Budget Act of 2018 § 41108) — collected proceeds INCLUDE penalties, interest, additions to tax, and additional amounts AND any proceeds from laws IRS is authorized to administer/enforce/investigate, INCLUDING CRIMINAL FINES AND CIVIL FORFEITURES, and violations of reporting requirements (e.g., FBAR under 31 USC § 5321)".to_string(),
        "26 USC § 7623(d) (Taxpayer First Act of 2019 § 1405) — anti-retaliation: no employer or officer/employee/contractor may discharge, demote, suspend, threaten, harass, or discriminate against whistleblower; remedies: reinstatement, DOUBLE back pay with interest, special damages, attorney fees".to_string(),
        "Tax Relief and Health Care Act of 2006 § 406 (Pub. L. 109-432, December 20, 2006) added § 7623(b) mandatory regime and created IRS Whistleblower Office; Form 211 (Application for Award for Original Information)".to_string(),
        "Treas. Reg. § 301.7623-4 — award starts at 15% baseline; analyzed for increase to 22% or 30% based on whistleblower's substantial contribution and other factors; IRM 25.2.2 governs Whistleblower Office procedures".to_string(),
    ];

    Section7623Result {
        tier,
        mandatory_thresholds_met,
        two_million_threshold_met: two_million_met,
        two_hundred_k_gross_income_threshold_met: two_hundred_k_met,
        min_award_percentage_bps: min_bps,
        max_award_percentage_bps: max_bps,
        award_cents,
        award_denied,
        tax_court_appeal_timely,
        anti_retaliation_violated,
        failure_reasons,
        citation: "26 USC § 7623(a)-(d); Tax Relief and Health Care Act of 2006 § 406 (Pub. L. 109-432); Bipartisan Budget Act of 2018 § 41108 (Pub. L. 115-123); Taxpayer First Act of 2019 § 1405(a) (Pub. L. 116-25); Treas. Reg. § 301.7623-1 to -4; IRM 25.2.2; Form 211; 31 USC § 5321",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mandatory_base() -> Section7623Input {
        Section7623Input {
            amount_in_dispute_cents: 300_000_000,
            taxpayer_gross_income_cents: 30_000_000,
            taxpayer_is_individual: true,
            information_acted_on_by_irs: true,
            collected_proceeds_cents: 250_000_000,
            proceeds_include_criminal_fines_civil_forfeitures: false,
            based_on_public_information: false,
            whistleblower_planned_or_initiated: false,
            whistleblower_convicted_of_role: false,
            award_percentage_bps: 2200,
            days_to_tax_court_appeal: 15,
            retaliation_occurred: false,
        }
    }

    #[test]
    fn three_million_dispute_three_hundred_k_income_mandatory_tier() {
        let r = check(&mandatory_base());
        assert_eq!(r.tier, AwardTier::MandatoryFullRange);
        assert!(r.mandatory_thresholds_met);
        assert_eq!(r.min_award_percentage_bps, 1500);
        assert_eq!(r.max_award_percentage_bps, 3000);
    }

    #[test]
    fn award_22_percent_of_2_5m_proceeds_is_550k() {
        let r = check(&mandatory_base());
        assert_eq!(r.award_cents, 55_000_000);
    }

    #[test]
    fn award_15_percent_floor_on_mandatory_band() {
        let mut i = mandatory_base();
        i.award_percentage_bps = 1500;
        let r = check(&i);
        assert_eq!(r.award_cents, 37_500_000);
    }

    #[test]
    fn award_30_percent_ceiling_on_mandatory_band() {
        let mut i = mandatory_base();
        i.award_percentage_bps = 3000;
        let r = check(&i);
        assert_eq!(r.award_cents, 75_000_000);
    }

    #[test]
    fn two_million_dispute_threshold_exactly_does_not_meet() {
        let mut i = mandatory_base();
        i.amount_in_dispute_cents = 200_000_000;
        let r = check(&i);
        assert!(!r.two_million_threshold_met);
        assert!(!r.mandatory_thresholds_met);
        assert_eq!(r.tier, AwardTier::DiscretionaryOnly);
    }

    #[test]
    fn two_million_one_cent_dispute_just_meets() {
        let mut i = mandatory_base();
        i.amount_in_dispute_cents = 200_000_001;
        let r = check(&i);
        assert!(r.two_million_threshold_met);
    }

    #[test]
    fn individual_with_under_200k_gross_income_drops_to_discretionary() {
        let mut i = mandatory_base();
        i.taxpayer_gross_income_cents = 20_000_000;
        let r = check(&i);
        assert!(!r.two_hundred_k_gross_income_threshold_met);
        assert_eq!(r.tier, AwardTier::DiscretionaryOnly);
    }

    #[test]
    fn corporate_taxpayer_skips_200k_gross_income_test() {
        let mut i = mandatory_base();
        i.taxpayer_is_individual = false;
        i.taxpayer_gross_income_cents = 0;
        let r = check(&i);
        assert!(r.two_hundred_k_gross_income_threshold_met);
        assert_eq!(r.tier, AwardTier::MandatoryFullRange);
    }

    #[test]
    fn information_not_acted_on_drops_to_discretionary() {
        let mut i = mandatory_base();
        i.information_acted_on_by_irs = false;
        let r = check(&i);
        assert_eq!(r.tier, AwardTier::DiscretionaryOnly);
    }

    #[test]
    fn public_information_caps_at_10_percent() {
        let mut i = mandatory_base();
        i.based_on_public_information = true;
        i.award_percentage_bps = 1000;
        let r = check(&i);
        assert_eq!(r.tier, AwardTier::ReducedPublicInformation);
        assert_eq!(r.max_award_percentage_bps, 1000);
        assert_eq!(r.award_cents, 25_000_000);
    }

    #[test]
    fn public_information_request_exceeding_10_pct_fails() {
        let mut i = mandatory_base();
        i.based_on_public_information = true;
        i.award_percentage_bps = 2000;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7623(b)(2)(A)") && f.contains("10%")));
    }

    #[test]
    fn whistleblower_planned_or_initiated_reduced_tier() {
        let mut i = mandatory_base();
        i.whistleblower_planned_or_initiated = true;
        let r = check(&i);
        assert_eq!(r.tier, AwardTier::ReducedPlannedInitiated);
    }

    #[test]
    fn whistleblower_convicted_denies_award_entirely() {
        let mut i = mandatory_base();
        i.whistleblower_convicted_of_role = true;
        let r = check(&i);
        assert_eq!(r.tier, AwardTier::DeniedConvicted);
        assert!(r.award_denied);
        assert_eq!(r.award_cents, 0);
        assert_eq!(r.max_award_percentage_bps, 0);
    }

    #[test]
    fn convicted_overrides_planned_and_public_info() {
        let mut i = mandatory_base();
        i.whistleblower_convicted_of_role = true;
        i.whistleblower_planned_or_initiated = true;
        i.based_on_public_information = true;
        let r = check(&i);
        assert_eq!(r.tier, AwardTier::DeniedConvicted);
    }

    #[test]
    fn tax_court_appeal_at_day_30_timely() {
        let mut i = mandatory_base();
        i.days_to_tax_court_appeal = 30;
        let r = check(&i);
        assert!(r.tax_court_appeal_timely);
    }

    #[test]
    fn tax_court_appeal_at_day_31_untimely() {
        let mut i = mandatory_base();
        i.days_to_tax_court_appeal = 31;
        let r = check(&i);
        assert!(!r.tax_court_appeal_timely);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7623(b)(4)") && f.contains("30 days")));
    }

    #[test]
    fn retaliation_triggers_taxpayer_first_act_violation() {
        let mut i = mandatory_base();
        i.retaliation_occurred = true;
        let r = check(&i);
        assert!(r.anti_retaliation_violated);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 7623(d)")
            && f.contains("Taxpayer First Act")
            && f.contains("DOUBLE back pay")));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&mandatory_base());
        assert!(r.citation.contains("§ 7623(a)-(d)"));
        assert!(r.citation.contains("Tax Relief and Health Care Act of 2006 § 406"));
        assert!(r.citation.contains("Pub. L. 109-432"));
        assert!(r.citation.contains("Bipartisan Budget Act of 2018 § 41108"));
        assert!(r.citation.contains("Pub. L. 115-123"));
        assert!(r.citation.contains("Taxpayer First Act of 2019 § 1405(a)"));
        assert!(r.citation.contains("Pub. L. 116-25"));
        assert!(r.citation.contains("Treas. Reg. § 301.7623"));
        assert!(r.citation.contains("IRM 25.2.2"));
        assert!(r.citation.contains("Form 211"));
        assert!(r.citation.contains("31 USC § 5321"));
    }

    #[test]
    fn note_pins_subsection_a_1867_origin() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(a)")
            && n.contains("1867 original")
            && n.contains("detecting underpayments")
            && n.contains("conniving")));
    }

    #[test]
    fn note_pins_subsection_b1_15_to_30_percent() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(1)")
            && n.contains("at least 15 percent but not more than 30 percent")));
    }

    #[test]
    fn note_pins_subsection_b2a_10_percent_public_info() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(2)(A)")
            && n.contains("up to 10 percent")
            && n.contains("news media")));
    }

    #[test]
    fn note_pins_subsection_b2b_planned_initiated() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(2)(B)")
            && n.contains("planned and initiated")));
    }

    #[test]
    fn note_pins_subsection_b3_conviction_no_award() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(3)")
            && n.contains("convicted of criminal conduct")
            && n.contains("no award shall be paid")));
    }

    #[test]
    fn note_pins_subsection_b4_30_day_tax_court_appeal() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(4)")
            && n.contains("30 days")
            && n.contains("Tax Court")));
    }

    #[test]
    fn note_pins_subsection_b5_two_million_two_hundred_k_thresholds() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(b)(5)")
            && n.contains("$2,000,000")
            && n.contains("$200,000")));
    }

    #[test]
    fn note_pins_subsection_c_bba_2018_criminal_fines_civil_forfeitures() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(c)")
            && n.contains("Bipartisan Budget Act of 2018 § 41108")
            && n.contains("CRIMINAL FINES AND CIVIL FORFEITURES")
            && n.contains("FBAR")));
    }

    #[test]
    fn note_pins_subsection_d_tfa_2019_anti_retaliation_double_back_pay() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7623(d)")
            && n.contains("Taxpayer First Act of 2019 § 1405")
            && n.contains("DOUBLE back pay")));
    }

    #[test]
    fn note_pins_2006_act_origin_form_211() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("Pub. L. 109-432")
            && n.contains("December 20, 2006")
            && n.contains("Form 211")));
    }

    #[test]
    fn note_pins_treas_reg_15_22_30_baseline() {
        let r = check(&mandatory_base());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 301.7623-4")
            && n.contains("15% baseline")
            && n.contains("22% or 30%")));
    }

    #[test]
    fn award_tier_truth_table_five_cells() {
        for (config, exp_tier) in [
            (
                (false, false, false, false, true, 30_000_000_u64, 300_000_000_u64),
                AwardTier::MandatoryFullRange,
            ),
            (
                (true, false, false, false, true, 30_000_000, 300_000_000),
                AwardTier::ReducedPublicInformation,
            ),
            (
                (false, true, false, false, true, 30_000_000, 300_000_000),
                AwardTier::ReducedPlannedInitiated,
            ),
            (
                (false, false, true, false, true, 30_000_000, 300_000_000),
                AwardTier::DeniedConvicted,
            ),
            (
                (false, false, false, false, true, 30_000_000, 100_000_000),
                AwardTier::DiscretionaryOnly,
            ),
        ] {
            let (pub_info, planned, convicted, _na, indiv, gi, dispute) = config;
            let i = Section7623Input {
                amount_in_dispute_cents: dispute,
                taxpayer_gross_income_cents: gi,
                taxpayer_is_individual: indiv,
                information_acted_on_by_irs: true,
                collected_proceeds_cents: 250_000_000,
                proceeds_include_criminal_fines_civil_forfeitures: false,
                based_on_public_information: pub_info,
                whistleblower_planned_or_initiated: planned,
                whistleblower_convicted_of_role: convicted,
                award_percentage_bps: 1500,
                days_to_tax_court_appeal: 15,
                retaliation_occurred: false,
            };
            let r = check(&i);
            assert_eq!(r.tier, exp_tier, "config={:?}", config);
        }
    }

    #[test]
    fn clamping_below_min_band_locks_at_floor() {
        let mut i = mandatory_base();
        i.award_percentage_bps = 1000;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7623(b)(1)") && f.contains("1500-3000 bps")));
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = mandatory_base();
        i.collected_proceeds_cents = u64::MAX;
        i.award_percentage_bps = 3000;
        let r = check(&i);
        let exp = u64::MAX.saturating_mul(3000) / 10_000;
        assert_eq!(r.award_cents, exp);
    }
}
