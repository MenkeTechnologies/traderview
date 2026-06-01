//! IRC § 6502 — Collection after assessment (CSED, Collection
//! Statute Expiration Date). Natural sibling to `section_6501`
//! (ASED — Assessment Statute Expiration Date). After the CSED
//! passes, the IRS is BARRED from collecting the assessed tax via
//! levy (§ 6331), lien (§ 6321), or court proceeding. Critical
//! trader-tax procedural protection — the strongest defensive
//! shield against IRS collection overreach on aged assessments.
//!
//! **Base period — § 6502(a)(1)**: 10 years from the date of
//! assessment. Most common CSED pathway.
//!
//! **Six independent suspension triggers** (each extends CSED):
//!
//! 1. **§ 6502(a)(2) installment agreement** — for requests on or
//!    after January 1, 2000, CSED suspended for the period agreed
//!    in writing PLUS 90 days after that period expires.
//!
//! 2. **§ 6331(k)(1) offer in compromise (OIC)** — CSED suspended
//!    from date OIC submitted until accepted, rejected, withdrawn,
//!    or returned. PLUS ADDITIONAL 30 DAYS if rejected.
//!
//! 3. **§ 6330(e)(1) CDP hearing (levy)** — CSED suspended from
//!    date CDP request received through hearing conclusion + any
//!    appeals. PLUS 90-day floor if less than 90 days remain on
//!    CSED at hearing conclusion.
//!
//! 4. **§ 6503(h) bankruptcy automatic stay** — CSED suspended
//!    for duration of the bankruptcy stay PLUS 6 MONTHS after
//!    stay terminates.
//!
//! 5. **§ 7508(a) military combat zone deferment** — CSED
//!    suspended for service in combat zone or contingency
//!    operation + 180 days after departure / hospitalization.
//!
//! 6. **§ 6503(c) taxpayer outside US continuously for 6+
//!    months** — CSED suspended for the period taxpayer is absent
//!    from US PLUS the time until return + 6 months.
//!
//! **Overlapping suspensions run CONCURRENTLY, not cumulatively**
//! (IRM 5.1.19.3.4). Two suspensions covering same period extend
//! CSED only once. Critical compliance trap — concurrent OIC + CDP
//! is the most common overlap.
//!
//! Trader-relevant: aggressive § 1256 mark-to-market positions,
//! § 988 currency reclassifications, and § 1202 QSBS holding-
//! period audits that produced assessments more than 10 years ago
//! may be barred from collection — unless suspension triggers
//! apply. Wash-sale loss reach-backs from § 6501 6-year extensions
//! also age out via § 6502.
//!
//! Citations: IRC § 6502(a)(1) (10-year base period); § 6502(a)(2)
//! (installment agreement + 90 days); § 6331(k)(1) (OIC + 30 days
//! if rejected); § 6330(e)(1) (CDP + 90-day floor); § 6503(h)
//! (bankruptcy + 6 months); § 7508(a) (combat zone + 180 days);
//! § 6503(c) (continuous absence + 6 months); IRM 5.1.19.3.4
//! (concurrent suspensions); IRM 8.21.5 (statutes on collection
//! cases).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6502Input {
    /// Whether an installment agreement under § 6159 was in effect
    /// at any point post-assessment, triggering § 6502(a)(2)
    /// suspension.
    pub installment_agreement_in_effect: bool,
    /// Whether an offer in compromise under § 7122 was pending
    /// post-assessment, triggering § 6331(k)(1) suspension.
    pub offer_in_compromise_pending: bool,
    /// Whether the offer in compromise was REJECTED, triggering
    /// additional 30-day extension under § 6331(k)(1).
    pub offer_in_compromise_rejected: bool,
    /// Whether a CDP hearing under § 6330 was requested,
    /// triggering § 6330(e)(1) suspension.
    pub cdp_hearing_requested: bool,
    /// Whether at CDP hearing conclusion fewer than 90 days
    /// remained on CSED, triggering 90-day floor.
    pub cdp_less_than_90_days_remaining: bool,
    /// Whether the taxpayer filed bankruptcy, triggering
    /// § 6503(h) automatic stay suspension.
    pub bankruptcy_stay_active: bool,
    /// Whether the taxpayer served in a combat zone or contingency
    /// operation, triggering § 7508(a) suspension.
    pub military_combat_zone_deferment: bool,
    /// Whether the taxpayer was continuously absent from the US
    /// for 6+ months post-assessment, triggering § 6503(c)
    /// suspension.
    pub taxpayer_continuously_absent_six_plus_months: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6502Result {
    /// Base CSED period in years (always 10 per § 6502(a)(1)).
    pub csed_base_period_years: u32,
    /// Whether any suspension trigger is engaged.
    pub any_suspension_engaged: bool,
    pub installment_agreement_suspension_engaged: bool,
    pub oic_suspension_engaged: bool,
    /// Whether the additional 30-day extension applies for
    /// rejected OIC.
    pub oic_post_decision_30_day_extension_engaged: bool,
    pub cdp_suspension_engaged: bool,
    /// Whether the 90-day floor extension applies (CDP hearing
    /// concluded with < 90 days remaining on CSED).
    pub cdp_90_day_floor_engaged: bool,
    pub bankruptcy_suspension_engaged: bool,
    pub military_deferment_engaged: bool,
    pub continuous_absence_suspension_engaged: bool,
    /// Whether multiple suspensions overlap, triggering the
    /// concurrent-not-cumulative IRM 5.1.19.3.4 warning.
    pub overlapping_suspensions_concurrent_warning: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6502Input) -> Section6502Result {
    let mut notes: Vec<String> = Vec::new();

    let oic_30_day = input.offer_in_compromise_pending && input.offer_in_compromise_rejected;
    let cdp_90_day_floor =
        input.cdp_hearing_requested && input.cdp_less_than_90_days_remaining;

    let active_count = [
        input.installment_agreement_in_effect,
        input.offer_in_compromise_pending,
        input.cdp_hearing_requested,
        input.bankruptcy_stay_active,
        input.military_combat_zone_deferment,
        input.taxpayer_continuously_absent_six_plus_months,
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    let any_suspension = active_count > 0;
    let overlapping = active_count >= 2;

    notes.push(
        "§ 6502(a)(1) — base CSED is 10 years from date of assessment; after CSED, IRS BARRED from collecting via levy (§ 6331), lien (§ 6321), or court proceeding"
            .to_string(),
    );

    if input.installment_agreement_in_effect {
        notes.push(
            "§ 6502(a)(2) — installment agreement suspends CSED for period agreed in writing PLUS 90 days after period expires (post-January 1, 2000 requests)"
                .to_string(),
        );
    }

    if input.offer_in_compromise_pending {
        notes.push(
            "§ 6331(k)(1) — OIC suspends CSED from submission date until accepted, rejected, withdrawn, or returned"
                .to_string(),
        );
        if input.offer_in_compromise_rejected {
            notes.push(
                "§ 6331(k)(1) — OIC rejected triggers ADDITIONAL 30-day CSED extension"
                    .to_string(),
            );
        }
    }

    if input.cdp_hearing_requested {
        notes.push(
            "§ 6330(e)(1) — CDP hearing request suspends CSED from receipt through hearing conclusion + appeals"
                .to_string(),
        );
        if input.cdp_less_than_90_days_remaining {
            notes.push(
                "§ 6330(e)(1) — 90-day floor engaged: if fewer than 90 days remain on CSED at hearing conclusion, statute extended to provide minimum 90 days"
                    .to_string(),
            );
        }
    }

    if input.bankruptcy_stay_active {
        notes.push(
            "§ 6503(h) — bankruptcy automatic stay suspends CSED for duration of stay PLUS 6 MONTHS after stay terminates"
                .to_string(),
        );
    }

    if input.military_combat_zone_deferment {
        notes.push(
            "§ 7508(a) — combat zone / contingency operation service suspends CSED + 180 days after departure or hospitalization"
                .to_string(),
        );
    }

    if input.taxpayer_continuously_absent_six_plus_months {
        notes.push(
            "§ 6503(c) — taxpayer continuously absent from US 6+ months suspends CSED for absence period PLUS until return + 6 months"
                .to_string(),
        );
    }

    if overlapping {
        notes.push(
            "IRM 5.1.19.3.4 — overlapping suspensions run CONCURRENTLY, not cumulatively; two suspensions covering same period extend CSED only once"
                .to_string(),
        );
    }

    notes.push(
        "IRM 8.21.5 — statutes on collection cases governs case-level CSED calculations; consult IRM for fact-specific application"
            .to_string(),
    );

    Section6502Result {
        csed_base_period_years: 10,
        any_suspension_engaged: any_suspension,
        installment_agreement_suspension_engaged: input.installment_agreement_in_effect,
        oic_suspension_engaged: input.offer_in_compromise_pending,
        oic_post_decision_30_day_extension_engaged: oic_30_day,
        cdp_suspension_engaged: input.cdp_hearing_requested,
        cdp_90_day_floor_engaged: cdp_90_day_floor,
        bankruptcy_suspension_engaged: input.bankruptcy_stay_active,
        military_deferment_engaged: input.military_combat_zone_deferment,
        continuous_absence_suspension_engaged: input.taxpayer_continuously_absent_six_plus_months,
        overlapping_suspensions_concurrent_warning: overlapping,
        citation: "IRC §§ 6502(a)(1), 6502(a)(2), 6331(k)(1), 6330(e)(1), 6503(h), 7508(a), 6503(c); IRM 5.1.19.3.4; IRM 8.21.5",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6502Input {
        Section6502Input {
            installment_agreement_in_effect: false,
            offer_in_compromise_pending: false,
            offer_in_compromise_rejected: false,
            cdp_hearing_requested: false,
            cdp_less_than_90_days_remaining: false,
            bankruptcy_stay_active: false,
            military_combat_zone_deferment: false,
            taxpayer_continuously_absent_six_plus_months: false,
        }
    }

    #[test]
    fn base_period_always_10_years() {
        let r = check(&base());
        assert_eq!(r.csed_base_period_years, 10);
    }

    #[test]
    fn no_suspensions_engaged_clean_base() {
        let r = check(&base());
        assert!(!r.any_suspension_engaged);
        assert!(!r.installment_agreement_suspension_engaged);
        assert!(!r.oic_suspension_engaged);
        assert!(!r.cdp_suspension_engaged);
        assert!(!r.bankruptcy_suspension_engaged);
        assert!(!r.military_deferment_engaged);
        assert!(!r.continuous_absence_suspension_engaged);
        assert!(!r.overlapping_suspensions_concurrent_warning);
    }

    #[test]
    fn base_period_note_describes_csed_consequence() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6502(a)(1)") && n.contains("BARRED from collecting")));
    }

    #[test]
    fn installment_agreement_engaged_with_90_day_extension_note() {
        let mut i = base();
        i.installment_agreement_in_effect = true;
        let r = check(&i);
        assert!(r.installment_agreement_suspension_engaged);
        assert!(r.any_suspension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6502(a)(2)") && n.contains("90 days after period expires")));
    }

    #[test]
    fn oic_pending_engages_suspension() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        let r = check(&i);
        assert!(r.oic_suspension_engaged);
        assert!(!r.oic_post_decision_30_day_extension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6331(k)(1)") && n.contains("OIC suspends CSED")));
    }

    #[test]
    fn oic_rejected_engages_30_day_extension() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        i.offer_in_compromise_rejected = true;
        let r = check(&i);
        assert!(r.oic_post_decision_30_day_extension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("ADDITIONAL 30-day")));
    }

    #[test]
    fn oic_rejected_without_pending_no_30_day_extension() {
        let mut i = base();
        i.offer_in_compromise_pending = false;
        i.offer_in_compromise_rejected = true;
        let r = check(&i);
        assert!(!r.oic_post_decision_30_day_extension_engaged);
    }

    #[test]
    fn cdp_hearing_engages_suspension() {
        let mut i = base();
        i.cdp_hearing_requested = true;
        let r = check(&i);
        assert!(r.cdp_suspension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6330(e)(1)") && n.contains("CDP hearing request")));
    }

    #[test]
    fn cdp_with_less_than_90_days_engages_90_day_floor() {
        let mut i = base();
        i.cdp_hearing_requested = true;
        i.cdp_less_than_90_days_remaining = true;
        let r = check(&i);
        assert!(r.cdp_90_day_floor_engaged);
        assert!(r.notes.iter().any(|n| n.contains("90-day floor engaged")));
    }

    #[test]
    fn cdp_90_day_floor_only_when_cdp_requested() {
        let mut i = base();
        i.cdp_less_than_90_days_remaining = true;
        let r = check(&i);
        assert!(!r.cdp_90_day_floor_engaged);
    }

    #[test]
    fn bankruptcy_engages_suspension_with_6_month_extension_note() {
        let mut i = base();
        i.bankruptcy_stay_active = true;
        let r = check(&i);
        assert!(r.bankruptcy_suspension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6503(h)") && n.contains("6 MONTHS")));
    }

    #[test]
    fn military_combat_zone_engages_with_180_day_note() {
        let mut i = base();
        i.military_combat_zone_deferment = true;
        let r = check(&i);
        assert!(r.military_deferment_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7508(a)") && n.contains("180 days")));
    }

    #[test]
    fn continuous_absence_engages_with_6_month_note() {
        let mut i = base();
        i.taxpayer_continuously_absent_six_plus_months = true;
        let r = check(&i);
        assert!(r.continuous_absence_suspension_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 6503(c)") && n.contains("absent from US 6+ months")));
    }

    #[test]
    fn overlapping_two_suspensions_engages_concurrent_warning() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        i.cdp_hearing_requested = true;
        let r = check(&i);
        assert!(r.overlapping_suspensions_concurrent_warning);
        assert!(r.notes.iter().any(|n| n.contains("IRM 5.1.19.3.4") && n.contains("CONCURRENTLY")));
    }

    #[test]
    fn single_suspension_no_overlap_warning() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        let r = check(&i);
        assert!(!r.overlapping_suspensions_concurrent_warning);
    }

    #[test]
    fn all_six_suspensions_engaged_simultaneously() {
        let i = Section6502Input {
            installment_agreement_in_effect: true,
            offer_in_compromise_pending: true,
            offer_in_compromise_rejected: false,
            cdp_hearing_requested: true,
            cdp_less_than_90_days_remaining: false,
            bankruptcy_stay_active: true,
            military_combat_zone_deferment: true,
            taxpayer_continuously_absent_six_plus_months: true,
        };
        let r = check(&i);
        assert!(r.installment_agreement_suspension_engaged);
        assert!(r.oic_suspension_engaged);
        assert!(r.cdp_suspension_engaged);
        assert!(r.bankruptcy_suspension_engaged);
        assert!(r.military_deferment_engaged);
        assert!(r.continuous_absence_suspension_engaged);
        assert!(r.overlapping_suspensions_concurrent_warning);
    }

    #[test]
    fn citation_pins_all_seven_authorities_and_irm() {
        let r = check(&base());
        assert!(r.citation.contains("§§ 6502(a)(1)"));
        assert!(r.citation.contains("6502(a)(2)"));
        assert!(r.citation.contains("6331(k)(1)"));
        assert!(r.citation.contains("6330(e)(1)"));
        assert!(r.citation.contains("6503(h)"));
        assert!(r.citation.contains("7508(a)"));
        assert!(r.citation.contains("6503(c)"));
        assert!(r.citation.contains("IRM 5.1.19.3.4"));
        assert!(r.citation.contains("IRM 8.21.5"));
    }

    #[test]
    fn irm_8_21_5_note_always_present() {
        let r = check(&base());
        assert!(r.notes.iter().any(|n| n.contains("IRM 8.21.5") && n.contains("statutes on collection cases")));
    }

    #[test]
    fn truth_table_six_independent_triggers_each_sets_flag() {
        let triggers: [(fn(&mut Section6502Input), fn(&Section6502Result) -> bool); 6] = [
            (|i| i.installment_agreement_in_effect = true, |r| r.installment_agreement_suspension_engaged),
            (|i| i.offer_in_compromise_pending = true, |r| r.oic_suspension_engaged),
            (|i| i.cdp_hearing_requested = true, |r| r.cdp_suspension_engaged),
            (|i| i.bankruptcy_stay_active = true, |r| r.bankruptcy_suspension_engaged),
            (|i| i.military_combat_zone_deferment = true, |r| r.military_deferment_engaged),
            (|i| i.taxpayer_continuously_absent_six_plus_months = true, |r| r.continuous_absence_suspension_engaged),
        ];

        for (setter, getter) in triggers {
            let mut i = base();
            setter(&mut i);
            let r = check(&i);
            assert!(getter(&r));
            assert!(r.any_suspension_engaged);
            assert!(!r.overlapping_suspensions_concurrent_warning);
        }
    }

    #[test]
    fn three_overlapping_suspensions_engage_warning() {
        let mut i = base();
        i.installment_agreement_in_effect = true;
        i.offer_in_compromise_pending = true;
        i.cdp_hearing_requested = true;
        let r = check(&i);
        assert!(r.overlapping_suspensions_concurrent_warning);
    }

    #[test]
    fn oic_30_day_extension_independent_of_cdp_90_day_floor() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        i.offer_in_compromise_rejected = true;
        i.cdp_hearing_requested = true;
        i.cdp_less_than_90_days_remaining = true;
        let r = check(&i);
        assert!(r.oic_post_decision_30_day_extension_engaged);
        assert!(r.cdp_90_day_floor_engaged);
        assert!(r.overlapping_suspensions_concurrent_warning);
    }

    #[test]
    fn rejected_oic_30_day_note_distinct_from_pending_note() {
        let mut i = base();
        i.offer_in_compromise_pending = true;
        i.offer_in_compromise_rejected = true;
        let r = check(&i);
        let suspends_count = r.notes.iter().filter(|n| n.contains("OIC suspends CSED")).count();
        let thirty_day_count = r.notes.iter().filter(|n| n.contains("ADDITIONAL 30-day")).count();
        assert_eq!(suspends_count, 1);
        assert_eq!(thirty_day_count, 1);
    }

    #[test]
    fn cdp_hearing_without_90_day_remaining_no_floor_note() {
        let mut i = base();
        i.cdp_hearing_requested = true;
        let r = check(&i);
        assert!(!r.cdp_90_day_floor_engaged);
        assert!(!r.notes.iter().any(|n| n.contains("90-day floor engaged")));
    }

    #[test]
    fn any_suspension_engaged_matches_individual_flags() {
        for trigger_idx in 0..6 {
            let mut i = base();
            match trigger_idx {
                0 => i.installment_agreement_in_effect = true,
                1 => i.offer_in_compromise_pending = true,
                2 => i.cdp_hearing_requested = true,
                3 => i.bankruptcy_stay_active = true,
                4 => i.military_combat_zone_deferment = true,
                5 => i.taxpayer_continuously_absent_six_plus_months = true,
                _ => unreachable!(),
            }
            let r = check(&i);
            assert!(r.any_suspension_engaged);
        }

        assert!(!check(&base()).any_suspension_engaged);
    }

    #[test]
    fn ten_year_base_immutable_across_all_inputs() {
        for trigger_idx in 0..6 {
            let mut i = base();
            match trigger_idx {
                0 => i.installment_agreement_in_effect = true,
                1 => i.offer_in_compromise_pending = true,
                2 => i.cdp_hearing_requested = true,
                3 => i.bankruptcy_stay_active = true,
                4 => i.military_combat_zone_deferment = true,
                5 => i.taxpayer_continuously_absent_six_plus_months = true,
                _ => unreachable!(),
            }
            assert_eq!(check(&i).csed_base_period_years, 10);
        }
    }

    #[test]
    fn overlap_warning_only_engages_with_2_plus_distinct_triggers() {
        let mut i_two = base();
        i_two.bankruptcy_stay_active = true;
        i_two.military_combat_zone_deferment = true;
        assert!(check(&i_two).overlapping_suspensions_concurrent_warning);

        let mut i_one = base();
        i_one.bankruptcy_stay_active = true;
        assert!(!check(&i_one).overlapping_suspensions_concurrent_warning);
    }
}
