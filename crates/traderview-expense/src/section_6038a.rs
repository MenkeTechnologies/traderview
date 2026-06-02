//! IRC § 6038A — Information returns by 25%-foreign-owned
//! domestic corporations and foreign-owned U.S. disregarded
//! entities (Form 5472). Companion to section_6038d (FATCA
//! Form 8938), section_6038 (controlled foreign partnership
//! / corporation reporting), section_6038b (transfers to
//! foreign entities), section_6038c (foreign corp engaged in
//! US trade/business), section_6501 (assessment SOL —
//! § 6038A non-filing TOLLS SOL indefinitely under
//! § 6501(c)(8)).
//!
//! Trader-critical because trading-LLC structures
//! frequently trigger § 6038A:
//! - Delaware/Wyoming/Nevada single-member LLC owned by
//!   foreign individual (treated as DOMESTIC CORPORATION
//!   for limited § 6038A purposes per Treas. Reg.
//!   § 1.6038A-1(c), effective tax years beginning
//!   2017-01-01 onward).
//! - US LLC jointly owned with foreign family members,
//!   business partners, or trusts (25% direct or indirect
//!   threshold).
//! - Foreign hedge fund / family office using US LLC as
//!   trading conduit.
//! - § 475(f) MTM election complicates related-party
//!   transaction tracking when intra-family-trust account
//!   transfers occur.
//!
//! **§ 6038A(a) Filing requirement** — every reporting
//! corporation that is a 25%-foreign-owned domestic
//! corporation OR a foreign corporation engaged in trade
//! or business within the United States SHALL furnish at
//! such time and in such manner as Secretary prescribes:
//! 1. Information described in § 6038A(b) concerning each
//!    related party; AND
//! 2. Such other information as Secretary requires by
//!    regulations.
//!
//! **§ 6038A(c)(1) 25%-foreign-ownership definition** — a
//! corporation is 25%-foreign-owned if at any time during
//! the taxable year, **one or more foreign persons own
//! (directly or indirectly) at least 25%** of:
//! 1. The total voting power of all classes of stock
//!    entitled to vote; OR
//! 2. The total value of all classes of stock.
//!
//! **Treas. Reg. § 1.6038A-1(c) Foreign-owned U.S.
//! disregarded entity (DRE) carveout** — for tax years
//! beginning on or after **January 1, 2017** and ending on
//! or after December 13, 2017, a foreign-owned U.S. DRE is
//! treated as an entity separate from its owner and
//! classified as a **DOMESTIC CORPORATION for limited
//! purposes of the requirements under § 6038A** that apply
//! to 25%-foreign-owned domestic corporations. Form 5472
//! filed as attachment to pro-forma Form 1120.
//!
//! **§ 6038A(c)(2) Reportable transaction definition** —
//! any transaction of a type specified by Secretary
//! between the reporting corporation and a related party
//! during the taxable year, including:
//! - Sales, assignments, leases, licenses
//! - Loans, advances, contributions
//! - Commissions, rents, royalties
//! - Use of property, including intangible property
//! - Reimbursements, amounts paid as compensation
//!
//! **§ 6038A(d)(1) Monetary penalty — base $25,000** — if
//! reporting corporation fails to furnish information
//! required under § 6038A(a) within time prescribed OR
//! fails to maintain or cause another to maintain records
//! required by § 6038A(b)(1)(C), such corporation shall
//! pay a penalty of **$25,000** for each taxable year with
//! respect to which such failure occurs.
//!
//! **§ 6038A(d)(2) Continuation penalty — $25,000 per
//! 30-day period after 90-day notification — UNCAPPED** —
//! if any failure described in § 6038A(d)(1) continues for
//! more than **90 days** after notification by Secretary,
//! the corporation shall pay an additional penalty of
//! **$25,000 for each 30-day period (or fraction thereof)**
//! during which the failure continues after expiration of
//! the 90-day period. **No statutory maximum** — penalty
//! continues to accrue until cure.
//!
//! **§ 6038A(d)(3) Reasonable cause exception** —
//! penalties may be abated upon showing of reasonable
//! cause (NOT willful neglect). Treas. Reg. § 1.6038A-4(b).
//!
//! **§ 6038A(b)(1)(C) Records retention** — reporting
//! corporation must maintain records as long as they may
//! be relevant or material to determining U.S. tax
//! treatment, but in no case less than the applicable
//! § 6501 statute of limitations on assessment and
//! collection.
//!
//! **§ 6501(c)(8) SOL tolling — UNLIMITED while § 6038A
//! non-compliant** — § 6501 ASED clock does not start
//! running until the required § 6038A information return
//! is filed; failure to file Form 5472 keeps the
//! assessment statute open INDEFINITELY.
//!
//! Citations: 26 USC § 6038A(a)-(e); 26 USC § 6501(c)(8);
//! Treas. Reg. § 1.6038A-1 to § 1.6038A-7; IRS Form 5472;
//! IRM 8.11.5 (International Penalties); IRM 20.1.9
//! (International Penalties); Form 1120 pro-forma
//! attachment requirement for foreign-owned DREs (effective
//! tax years beginning 2017-01-01 per T.D. 9796).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Standard 25%-foreign-owned US corporation.
    UsCorporation,
    /// Foreign-owned US single-member LLC classified as
    /// disregarded entity — treated as domestic corp for
    /// § 6038A purposes per Treas. Reg. § 1.6038A-1(c)
    /// (effective TY beginning 2017-01-01).
    ForeignOwnedSingleMemberLlc,
    /// Foreign corporation engaged in trade or business
    /// within the United States.
    ForeignCorpEngagedInUsTradeOrBusiness,
    /// Non-qualifying entity (not subject to § 6038A).
    NotSubjectToSection6038A,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6038aInput {
    pub entity_type: EntityType,
    /// Maximum foreign ownership percentage in basis
    /// points (e.g., 2500 = 25%) at ANY time during tax
    /// year, direct + indirect aggregate.
    pub max_foreign_ownership_bps: u32,
    /// Tax year of determination (DRE carveout requires
    /// TY beginning ≥ 2017).
    pub tax_year: u32,
    /// Number of reportable transactions during the tax
    /// year (sales, loans, royalties, licenses, etc.).
    pub reportable_transaction_count: u32,
    /// Whether Form 5472 was filed for the tax year.
    pub form_5472_filed: bool,
    /// Whether required § 6038A(b)(1)(C) related-party
    /// transaction records were maintained.
    pub records_maintained: bool,
    /// Days since IRS § 6038A(d)(2) notification of
    /// failure (continuation-penalty clock).
    pub days_since_irs_notification: u32,
    /// Whether reasonable cause defense applies (NOT
    /// willful neglect) under § 6038A(d)(3) and Treas.
    /// Reg. § 1.6038A-4(b).
    pub reasonable_cause_engaged: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6038aResult {
    pub entity_type: EntityType,
    pub subject_to_section_6038a: bool,
    pub twenty_five_percent_threshold_met: bool,
    pub form_5472_filing_required: bool,
    pub base_penalty_cents: u64,
    pub continuation_penalty_cents: u64,
    pub total_penalty_cents: u64,
    pub section_6501_c8_sol_tolled: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6038aInput) -> Section6038aResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let twenty_five_percent_threshold_met = input.max_foreign_ownership_bps >= 2500;

    let dre_carveout_engaged = matches!(input.entity_type, EntityType::ForeignOwnedSingleMemberLlc)
        && input.tax_year >= 2017;

    let subject_to_section_6038a = match input.entity_type {
        EntityType::UsCorporation => twenty_five_percent_threshold_met,
        EntityType::ForeignOwnedSingleMemberLlc => dre_carveout_engaged,
        EntityType::ForeignCorpEngagedInUsTradeOrBusiness => true,
        EntityType::NotSubjectToSection6038A => false,
    };

    let has_reportable_transactions = input.reportable_transaction_count > 0;
    let form_5472_filing_required = subject_to_section_6038a
        && (has_reportable_transactions
            || matches!(
                input.entity_type,
                EntityType::ForeignOwnedSingleMemberLlc
            ));

    let mut base_penalty_cents: u64 = 0;
    let mut continuation_penalty_cents: u64 = 0;

    if form_5472_filing_required && (!input.form_5472_filed || !input.records_maintained) {
        if !input.reasonable_cause_engaged {
            base_penalty_cents = 2_500_000;
            failure_reasons.push(
                "26 USC § 6038A(d)(1) — failure to file Form 5472 OR maintain § 6038A(b)(1)(C) related-party transaction records triggers $25,000 base penalty per taxable year per reporting corporation".to_string(),
            );

            if input.days_since_irs_notification > 90 {
                let days_beyond_90 = input.days_since_irs_notification.saturating_sub(90);
                let thirty_day_periods = days_beyond_90.div_ceil(30);
                continuation_penalty_cents =
                    2_500_000_u64.saturating_mul(thirty_day_periods as u64);
                failure_reasons.push(format!(
                    "26 USC § 6038A(d)(2) — continuation penalty $25,000 per 30-day period (or fraction thereof) AFTER 90-day IRS notification — UNCAPPED; {} days past notification = {} thirty-day periods accrued",
                    input.days_since_irs_notification, thirty_day_periods
                ));
            }
        } else {
            failure_reasons.push(
                "26 USC § 6038A(d)(3) + Treas. Reg. § 1.6038A-4(b) — reasonable cause defense engaged; penalty abatement available if NOT willful neglect".to_string(),
            );
        }
    }

    let section_6501_c8_sol_tolled = form_5472_filing_required && !input.form_5472_filed;
    if section_6501_c8_sol_tolled {
        failure_reasons.push(
            "26 USC § 6501(c)(8) — § 6501 assessment statute of limitations does NOT start running until required § 6038A information return is filed; failure to file Form 5472 keeps assessment SOL OPEN INDEFINITELY".to_string(),
        );
    }

    let total_penalty_cents = base_penalty_cents.saturating_add(continuation_penalty_cents);

    let notes: Vec<String> = vec![
        "26 USC § 6038A(a) — every 25%-foreign-owned domestic corporation OR foreign corporation engaged in US trade or business SHALL furnish information about each related party and reportable transactions on Form 5472".to_string(),
        "26 USC § 6038A(c)(1) — 25%-foreign-ownership = one or more foreign persons own (directly or indirectly) at least 25% of total voting power OR total value at ANY TIME during the taxable year".to_string(),
        "Treas. Reg. § 1.6038A-1(c) — foreign-owned U.S. single-member LLC classified as DISREGARDED ENTITY is treated as DOMESTIC CORPORATION for limited purposes of § 6038A; effective tax years beginning on or after January 1, 2017 per T.D. 9796 (December 13, 2016)".to_string(),
        "26 USC § 6038A(c)(2) — reportable transactions include sales, assignments, leases, licenses, loans, advances, contributions, commissions, rents, royalties, use of intangible property, reimbursements, compensation".to_string(),
        "26 USC § 6038A(d)(1) — BASE PENALTY $25,000 per taxable year per reporting corporation for failure to file Form 5472 OR failure to maintain § 6038A(b)(1)(C) records".to_string(),
        "26 USC § 6038A(d)(2) — CONTINUATION PENALTY $25,000 per 30-day period (or fraction thereof) AFTER 90 DAYS following IRS notification — NO MAXIMUM CAP; accrues until cure by proper filing".to_string(),
        "26 USC § 6038A(d)(3) + Treas. Reg. § 1.6038A-4(b) — reasonable cause defense (NOT willful neglect) may abate penalty".to_string(),
        "26 USC § 6038A(b)(1)(C) — records retention: as long as relevant or material to determining US tax treatment, but never less than § 6501 applicable statute of limitations on assessment and collection".to_string(),
        "26 USC § 6501(c)(8) — assessment SOL does NOT start running until required § 6038A information return is filed; failure to file Form 5472 keeps § 6501 ASED OPEN INDEFINITELY for ENTIRE TAX YEAR".to_string(),
        "Form 5472 filed as attachment to pro-forma Form 1120 for foreign-owned single-member LLC disregarded entities (per T.D. 9796 effective TY beginning 2017-01-01)".to_string(),
        "IRM 8.11.5 (International Penalties) + IRM 20.1.9 (International Penalties) — internal IRS administrative guidance on § 6038A penalty assessment, notification, and abatement procedures".to_string(),
    ];

    Section6038aResult {
        entity_type: input.entity_type,
        subject_to_section_6038a,
        twenty_five_percent_threshold_met,
        form_5472_filing_required,
        base_penalty_cents,
        continuation_penalty_cents,
        total_penalty_cents,
        section_6501_c8_sol_tolled,
        failure_reasons,
        citation: "26 USC § 6038A(a)-(e); 26 USC § 6501(c)(8); Treas. Reg. § 1.6038A-1 to § 1.6038A-7; T.D. 9796 (December 13, 2016); IRS Form 5472; IRM 8.11.5; IRM 20.1.9",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn us_corp_base() -> Section6038aInput {
        Section6038aInput {
            entity_type: EntityType::UsCorporation,
            max_foreign_ownership_bps: 3000,
            tax_year: 2026,
            reportable_transaction_count: 5,
            form_5472_filed: true,
            records_maintained: true,
            days_since_irs_notification: 0,
            reasonable_cause_engaged: false,
        }
    }

    #[test]
    fn us_corp_30_percent_foreign_owned_subject_to_section() {
        let r = check(&us_corp_base());
        assert!(r.subject_to_section_6038a);
        assert!(r.twenty_five_percent_threshold_met);
        assert!(r.form_5472_filing_required);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn us_corp_24_99_percent_does_not_meet_threshold() {
        let mut i = us_corp_base();
        i.max_foreign_ownership_bps = 2499;
        let r = check(&i);
        assert!(!r.twenty_five_percent_threshold_met);
        assert!(!r.subject_to_section_6038a);
        assert!(!r.form_5472_filing_required);
    }

    #[test]
    fn us_corp_exactly_25_percent_meets_threshold() {
        let mut i = us_corp_base();
        i.max_foreign_ownership_bps = 2500;
        let r = check(&i);
        assert!(r.twenty_five_percent_threshold_met);
        assert!(r.subject_to_section_6038a);
    }

    #[test]
    fn us_corp_failure_to_file_base_penalty_25k() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 2_500_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038A(d)(1)")
            && f.contains("$25,000 base penalty")));
    }

    #[test]
    fn us_corp_failure_to_maintain_records_triggers_penalty() {
        let mut i = us_corp_base();
        i.records_maintained = false;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 2_500_000);
    }

    #[test]
    fn us_corp_continuation_penalty_one_30_day_period_25k() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 120;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 2_500_000);
        assert_eq!(r.continuation_penalty_cents, 2_500_000);
        assert_eq!(r.total_penalty_cents, 5_000_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038A(d)(2)")
            && f.contains("UNCAPPED")));
    }

    #[test]
    fn us_corp_continuation_penalty_three_30_day_periods_75k() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 180;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 7_500_000);
        assert_eq!(r.total_penalty_cents, 10_000_000);
    }

    #[test]
    fn us_corp_continuation_penalty_fraction_counts_as_full_period() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 91;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 2_500_000);
    }

    #[test]
    fn us_corp_no_continuation_at_90_day_boundary_exactly() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 90;
        let r = check(&i);
        assert_eq!(r.continuation_penalty_cents, 0);
        assert_eq!(r.total_penalty_cents, 2_500_000);
    }

    #[test]
    fn reasonable_cause_zeros_base_and_continuation_penalty() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = 200;
        i.reasonable_cause_engaged = true;
        let r = check(&i);
        assert_eq!(r.base_penalty_cents, 0);
        assert_eq!(r.continuation_penalty_cents, 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6038A(d)(3)")
            && f.contains("reasonable cause")));
    }

    #[test]
    fn foreign_owned_single_member_llc_post_2017_subject() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::ForeignOwnedSingleMemberLlc;
        i.tax_year = 2026;
        let r = check(&i);
        assert!(r.subject_to_section_6038a);
        assert!(r.form_5472_filing_required);
    }

    #[test]
    fn foreign_owned_single_member_llc_pre_2017_not_subject() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::ForeignOwnedSingleMemberLlc;
        i.tax_year = 2016;
        let r = check(&i);
        assert!(!r.subject_to_section_6038a);
    }

    #[test]
    fn foreign_owned_single_member_llc_2017_boundary_subject() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::ForeignOwnedSingleMemberLlc;
        i.tax_year = 2017;
        let r = check(&i);
        assert!(r.subject_to_section_6038a);
    }

    #[test]
    fn foreign_owned_dre_requires_filing_even_without_reportable_transactions() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::ForeignOwnedSingleMemberLlc;
        i.tax_year = 2026;
        i.reportable_transaction_count = 0;
        let r = check(&i);
        assert!(r.form_5472_filing_required);
    }

    #[test]
    fn foreign_corp_engaged_in_us_business_subject_to_section() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::ForeignCorpEngagedInUsTradeOrBusiness;
        let r = check(&i);
        assert!(r.subject_to_section_6038a);
    }

    #[test]
    fn not_subject_to_entity_type_no_obligation() {
        let mut i = us_corp_base();
        i.entity_type = EntityType::NotSubjectToSection6038A;
        let r = check(&i);
        assert!(!r.subject_to_section_6038a);
        assert!(!r.form_5472_filing_required);
    }

    #[test]
    fn section_6501_c8_sol_tolled_on_non_filing() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        let r = check(&i);
        assert!(r.section_6501_c8_sol_tolled);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6501(c)(8)")
            && f.contains("OPEN INDEFINITELY")));
    }

    #[test]
    fn section_6501_c8_sol_not_tolled_when_form_filed() {
        let r = check(&us_corp_base());
        assert!(!r.section_6501_c8_sol_tolled);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&us_corp_base());
        assert!(r.citation.contains("§ 6038A(a)-(e)"));
        assert!(r.citation.contains("§ 6501(c)(8)"));
        assert!(r.citation.contains("Treas. Reg. § 1.6038A-1 to § 1.6038A-7"));
        assert!(r.citation.contains("T.D. 9796"));
        assert!(r.citation.contains("December 13, 2016"));
        assert!(r.citation.contains("Form 5472"));
        assert!(r.citation.contains("IRM 8.11.5"));
        assert!(r.citation.contains("IRM 20.1.9"));
    }

    #[test]
    fn note_pins_subsection_a_filing_requirement() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(a)")
            && n.contains("25%-foreign-owned")
            && n.contains("Form 5472")));
    }

    #[test]
    fn note_pins_subsection_c1_25_percent_definition() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(c)(1)")
            && n.contains("voting power OR total value")
            && n.contains("ANY TIME")));
    }

    #[test]
    fn note_pins_dre_carveout_2017_tds_9796() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.6038A-1(c)")
            && n.contains("DISREGARDED ENTITY")
            && n.contains("January 1, 2017")
            && n.contains("T.D. 9796")));
    }

    #[test]
    fn note_pins_subsection_c2_reportable_transactions() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(c)(2)")
            && n.contains("sales, assignments")
            && n.contains("royalties")));
    }

    #[test]
    fn note_pins_subsection_d1_25k_base_penalty() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(d)(1)")
            && n.contains("BASE PENALTY $25,000")));
    }

    #[test]
    fn note_pins_subsection_d2_continuation_uncapped() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(d)(2)")
            && n.contains("CONTINUATION PENALTY $25,000")
            && n.contains("90 DAYS")
            && n.contains("NO MAXIMUM CAP")));
    }

    #[test]
    fn note_pins_subsection_d3_reasonable_cause() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(d)(3)")
            && n.contains("Treas. Reg. § 1.6038A-4(b)")
            && n.contains("reasonable cause")));
    }

    #[test]
    fn note_pins_records_retention_b1c() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6038A(b)(1)(C)")
            && n.contains("records retention")
            && n.contains("§ 6501")));
    }

    #[test]
    fn note_pins_6501_c8_sol_tolling() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6501(c)(8)")
            && n.contains("OPEN INDEFINITELY")));
    }

    #[test]
    fn note_pins_form_1120_pro_forma_attachment() {
        let r = check(&us_corp_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 5472 filed as attachment to pro-forma Form 1120")));
    }

    #[test]
    fn entity_type_truth_table_four_cells() {
        for (et, expected_subject) in [
            (EntityType::UsCorporation, true),
            (EntityType::ForeignOwnedSingleMemberLlc, true),
            (EntityType::ForeignCorpEngagedInUsTradeOrBusiness, true),
            (EntityType::NotSubjectToSection6038A, false),
        ] {
            let mut i = us_corp_base();
            i.entity_type = et;
            i.max_foreign_ownership_bps = 5000;
            i.tax_year = 2026;
            let r = check(&i);
            assert_eq!(r.subject_to_section_6038a, expected_subject, "entity={:?}", et);
        }
    }

    #[test]
    fn ownership_threshold_truth_table() {
        for (bps, expected_meets) in [
            (0_u32, false),
            (2499_u32, false),
            (2500_u32, true),
            (2501_u32, true),
            (5000_u32, true),
            (10_000_u32, true),
        ] {
            let mut i = us_corp_base();
            i.max_foreign_ownership_bps = bps;
            let r = check(&i);
            assert_eq!(r.twenty_five_percent_threshold_met, expected_meets, "bps={}", bps);
        }
    }

    #[test]
    fn defensive_overflow_clamped_with_saturating_mul() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.days_since_irs_notification = u32::MAX;
        let r = check(&i);
        let _ = r.total_penalty_cents;
        assert!(r.base_penalty_cents == 2_500_000);
    }

    #[test]
    fn multiple_failure_reasons_stack() {
        let mut i = us_corp_base();
        i.form_5472_filed = false;
        i.records_maintained = false;
        i.days_since_irs_notification = 200;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 2);
    }
}
