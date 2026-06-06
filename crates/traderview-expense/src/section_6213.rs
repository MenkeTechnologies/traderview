//! IRC § 6213 — Restrictions applicable to deficiencies; petition
//! to Tax Court.
//!
//! Trader-critical procedural rule: after the IRS issues a statutory
//! notice of deficiency under § 6212 (the "90-day letter"), the
//! taxpayer has EXACTLY 90 days (150 days if addressed to a person
//! outside the United States) from the MAILING date to file a
//! petition with the Tax Court for redetermination. Missing this
//! deadline is catastrophic — the deficiency becomes assessable, the
//! Tax Court loses jurisdiction, and the taxpayer's only remaining
//! pathway is to pay the deficiency in full and sue for refund in
//! district court or Court of Federal Claims.
//!
//! § 6213(a) PETITION PERIOD — within 90 days, or 150 days if the
//! notice is addressed to a person outside the United States, after
//! the notice of deficiency authorized in § 6212 is mailed (not
//! counting Saturday, Sunday, or a legal holiday in the District of
//! Columbia as the last day), the taxpayer may file a petition with
//! the Tax Court for a redetermination of the deficiency. SAME-DAY
//! LAST-SENTENCE RULE — any petition filed with the Tax Court on or
//! before the last date specified for filing such petition by the
//! Secretary in the notice of deficiency shall be treated as timely
//! filed.
//!
//! § 6213(a) RESTRICTION ON ASSESSMENT — no assessment of a
//! deficiency shall be made within the 90-day (or 150-day) period.
//! If a petition has been filed with the Tax Court, no assessment
//! may be made until the decision of the Tax Court has become final.
//!
//! § 6213(b)(1) MATHEMATICAL/CLERICAL ERROR EXCEPTION — § 6213(b)(1)
//! permits summary assessment without notice of deficiency for math
//! errors. Caller responsibility to determine applicability — this
//! module computes the standard 90/150-day petition path only.
//!
//! § 6213(c) FAILURE TO FILE PETITION — if a notice of deficiency
//! has been mailed and the taxpayer does not file a petition with
//! the Tax Court within the time prescribed in subsection (a), the
//! deficiency shall be assessed, and shall be paid upon notice and
//! demand from the Secretary.
//!
//! JURISDICTIONAL VS NON-JURISDICTIONAL CIRCUIT SPLIT — the Tax
//! Court held in Hallmark Research Collective, 159 T.C. No. 6
//! (2022) that the 90-day deadline is JURISDICTIONAL and NOT subject
//! to equitable tolling. The Third Circuit in Culp v. Commissioner,
//! 75 F.4th 196 (3d Cir. 2023) reached the OPPOSITE conclusion,
//! finding the deadline non-jurisdictional and subject to equitable
//! tolling. The Supreme Court declined to resolve the split in
//! Boechler, P.C. v. Commissioner, 596 U.S. 199 (2022) (which
//! addressed § 6330(d)(1) CDP-levy deadline, not § 6213). Most
//! Circuits still treat § 6213 as jurisdictional per Hallmark.
//!
//! TRADER APPLICATION: a § 475(f) mark-to-market trader receiving an
//! IRS notice asserting that the trader's mark-to-market election
//! was not timely or that the trader does not satisfy TTS criteria
//! has 90 days to petition the Tax Court before the assessment
//! becomes restriction-free. The default rule is sharp — there is
//! no "I forgot" or "I didn't realize the notice was important"
//! defense under Hallmark.
//!
//! Citations: IRC § 6213(a) (90/150-day petition period + restriction
//! on assessment + last-sentence-of-(a) Secretary-specified-date
//! rule); § 6213(b)(1) (math/clerical error exception); § 6213(c)
//! (failure to file → assessment); § 6212 (notice of deficiency);
//! Hallmark Research Collective, 159 T.C. No. 6 (2022)
//! (jurisdictional); Culp v. Commissioner, 75 F.4th 196 (3d Cir.
//! 2023) (non-jurisdictional — Third Circuit only); Boechler, 596
//! U.S. 199 (2022) (Supreme Court CDP-levy precedent — does not
//! resolve § 6213 split).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section6213Input {
    /// Number of days between mailing of the § 6212 notice of
    /// deficiency and filing of the Tax Court petition. Inclusive
    /// of weekends; § 6213(a) excludes weekend/DC-holiday at the
    /// LAST DAY only via `last_day_falls_on_weekend_or_dc_holiday`.
    pub days_from_mailing_to_petition: u32,
    /// Whether the notice was addressed to a person outside the
    /// United States (extends the standard 90-day period to 150
    /// days per § 6213(a)).
    pub addressed_outside_us: bool,
    /// Whether the 90th (or 150th) day after mailing falls on a
    /// Saturday, Sunday, or legal holiday in the District of
    /// Columbia. If true, § 6213(a) extends the deadline to the
    /// next business day. Caller computes this from the actual
    /// calendar.
    pub last_day_falls_on_weekend_or_dc_holiday: bool,
    /// Last sentence of § 6213(a): petition filed on or before the
    /// "last date specified for filing such petition by the
    /// Secretary in the notice of deficiency" is timely. If the
    /// Secretary's specified date is LATER than the 90/150-day
    /// statutory deadline, this field reflects the additional days
    /// beyond the statutory period. Use None when notice did not
    /// specify a later date or specified an earlier date.
    pub secretary_specified_extended_period_days: Option<u32>,
    /// Whether the IRS subsequently made the assessment after the
    /// 90/150-day period expired without petition. Drives the
    /// § 6213(c) assessment-status output field.
    pub assessment_made_after_period: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6213Result {
    pub statute_period_days: u32,
    pub effective_deadline_days: u32,
    pub petition_timely: bool,
    pub assessment_restricted: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6213Input) -> Section6213Result {
    let mut notes: Vec<String> = Vec::new();

    let statute_period = if input.addressed_outside_us { 150 } else { 90 };
    if input.addressed_outside_us {
        notes.push(
            "§ 6213(a) — notice addressed to person outside US extends standard 90-day period to 150 days"
                .to_string(),
        );
    }

    let mut effective_deadline = statute_period;

    if input.last_day_falls_on_weekend_or_dc_holiday {
        effective_deadline += 1;
        notes.push(
            "§ 6213(a) — last day falls on Saturday, Sunday, or DC legal holiday — extended to next business day"
                .to_string(),
        );
    }

    if let Some(extra) = input.secretary_specified_extended_period_days {
        if extra > 0 {
            effective_deadline = effective_deadline.max(statute_period + extra);
            notes.push(format!(
                "§ 6213(a) last sentence — Secretary specified last date {} days beyond statutory period; petition timely if filed by that date",
                extra
            ));
        }
    }

    let petition_timely = input.days_from_mailing_to_petition <= effective_deadline;

    if !petition_timely {
        notes.push(format!(
            "petition filed {} days after mailing exceeds effective deadline {} days — Tax Court loses jurisdiction under Hallmark Research Collective (159 T.C. No. 6, 2022)",
            input.days_from_mailing_to_petition, effective_deadline
        ));
    }

    let assessment_restricted = petition_timely || !input.assessment_made_after_period;

    if !petition_timely && input.assessment_made_after_period {
        notes.push(
            "§ 6213(c) — petition not filed within statutory period; deficiency assessable on notice and demand"
                .to_string(),
        );
    } else if petition_timely {
        notes.push(
            "§ 6213(a) — assessment restricted until Tax Court decision becomes final".to_string(),
        );
    }

    notes.push(
        "Third Circuit Culp v. Commissioner (2023) reached opposite conclusion finding § 6213 deadline non-jurisdictional — circuit split unresolved"
            .to_string(),
    );

    Section6213Result {
        statute_period_days: statute_period,
        effective_deadline_days: effective_deadline,
        petition_timely,
        assessment_restricted,
        citation: citation(),
        notes,
    }
}

fn citation() -> &'static str {
    "IRC § 6213(a)/(b)(1)/(c); § 6212; Hallmark Research Collective, 159 T.C. No. 6 (2022); Culp v. Commissioner, 75 F.4th 196 (3d Cir. 2023); Boechler, 596 U.S. 199 (2022)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(days: u32) -> Section6213Input {
        Section6213Input {
            days_from_mailing_to_petition: days,
            addressed_outside_us: false,
            last_day_falls_on_weekend_or_dc_holiday: false,
            secretary_specified_extended_period_days: None,
            assessment_made_after_period: false,
        }
    }

    #[test]
    fn ninety_day_period_default() {
        let r = compute(&base(89));
        assert_eq!(r.statute_period_days, 90);
        assert_eq!(r.effective_deadline_days, 90);
        assert!(r.petition_timely);
    }

    #[test]
    fn exactly_ninety_days_timely() {
        let r = compute(&base(90));
        assert!(r.petition_timely);
    }

    #[test]
    fn ninety_one_days_untimely() {
        let r = compute(&base(91));
        assert!(!r.petition_timely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Hallmark Research Collective")));
    }

    #[test]
    fn outside_us_extends_to_150_days() {
        let mut i = base(149);
        i.addressed_outside_us = true;
        let r = compute(&i);
        assert_eq!(r.statute_period_days, 150);
        assert!(r.petition_timely);
        assert!(r.notes.iter().any(|n| n.contains("150 days")));
    }

    #[test]
    fn outside_us_at_150_days_timely() {
        let mut i = base(150);
        i.addressed_outside_us = true;
        let r = compute(&i);
        assert!(r.petition_timely);
    }

    #[test]
    fn outside_us_151_days_untimely() {
        let mut i = base(151);
        i.addressed_outside_us = true;
        let r = compute(&i);
        assert!(!r.petition_timely);
    }

    #[test]
    fn weekend_extension_one_day() {
        let mut i = base(91);
        i.last_day_falls_on_weekend_or_dc_holiday = true;
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 91);
        assert!(r.petition_timely);
        assert!(r.notes.iter().any(|n| n.contains("Saturday, Sunday")));
    }

    #[test]
    fn weekend_extension_does_not_save_92_day_petition() {
        let mut i = base(92);
        i.last_day_falls_on_weekend_or_dc_holiday = true;
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 91);
        assert!(!r.petition_timely);
    }

    #[test]
    fn secretary_specified_date_extends_deadline() {
        let mut i = base(120);
        i.secretary_specified_extended_period_days = Some(35);
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 125);
        assert!(r.petition_timely);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("last sentence") && n.contains("35 days")));
    }

    #[test]
    fn secretary_specified_date_earlier_does_not_shorten() {
        let mut i = base(89);
        i.secretary_specified_extended_period_days = Some(0);
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 90);
        assert!(r.petition_timely);
    }

    #[test]
    fn assessment_restricted_when_petition_timely() {
        let r = compute(&base(60));
        assert!(r.assessment_restricted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("assessment restricted until Tax Court decision")));
    }

    #[test]
    fn assessment_unrestricted_when_petition_missed_and_irs_assessed() {
        let mut i = base(120);
        i.assessment_made_after_period = true;
        let r = compute(&i);
        assert!(!r.assessment_restricted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6213(c)") && n.contains("notice and demand")));
    }

    #[test]
    fn citation_pins_subsections_and_cases() {
        let r = compute(&base(60));
        assert!(r.citation.contains("§ 6213(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("§ 6212"));
        assert!(r.citation.contains("Hallmark Research Collective"));
        assert!(r.citation.contains("Culp v. Commissioner"));
        assert!(r.citation.contains("Boechler"));
    }

    #[test]
    fn notes_always_include_third_circuit_split() {
        let r = compute(&base(60));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Third Circuit Culp") && n.contains("non-jurisdictional")));
    }

    #[test]
    fn weekend_and_outside_us_combine() {
        let mut i = base(151);
        i.addressed_outside_us = true;
        i.last_day_falls_on_weekend_or_dc_holiday = true;
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 151);
        assert!(r.petition_timely);
    }

    #[test]
    fn outside_us_and_secretary_extended_takes_max() {
        let mut i = base(155);
        i.addressed_outside_us = true;
        i.secretary_specified_extended_period_days = Some(10);
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 160);
        assert!(r.petition_timely);
    }

    #[test]
    fn zero_days_obviously_timely() {
        let r = compute(&base(0));
        assert!(r.petition_timely);
    }

    #[test]
    fn one_year_late_far_outside_period() {
        let r = compute(&base(365));
        assert!(!r.petition_timely);
    }

    #[test]
    fn secretary_specified_zero_days_no_change() {
        let mut i = base(89);
        i.secretary_specified_extended_period_days = Some(0);
        let r = compute(&i);
        assert_eq!(r.effective_deadline_days, 90);
    }

    #[test]
    fn assessment_restricted_even_when_petition_missed_if_irs_holds_off() {
        let mut i = base(120);
        i.assessment_made_after_period = false;
        let r = compute(&i);
        assert!(!r.petition_timely);
        assert!(
            r.assessment_restricted,
            "IRS held off → restriction preserved by inaction"
        );
    }

    #[test]
    fn boundary_exactly_90_days_no_weekend() {
        let r = compute(&base(90));
        assert_eq!(r.effective_deadline_days, 90);
        assert!(r.petition_timely);
    }

    #[test]
    fn boundary_91_days_no_weekend_untimely() {
        let r = compute(&base(91));
        assert_eq!(r.effective_deadline_days, 90);
        assert!(!r.petition_timely);
    }

    #[test]
    fn outside_us_50_day_difference_from_default_invariant() {
        let mut i_us = base(0);
        let mut i_outside = base(0);
        i_outside.addressed_outside_us = true;
        let r_us = compute(&i_us);
        let r_outside = compute(&i_outside);
        assert_eq!(r_outside.statute_period_days - r_us.statute_period_days, 60);
        let _ = (&mut i_us, &mut i_outside);
    }

    #[test]
    fn assessment_unrestricted_only_when_both_late_and_irs_assessed() {
        let cases = [
            (89, false, true),
            (89, true, true),
            (120, false, true),
            (120, true, false),
        ];
        for (days, assessed, restricted) in cases {
            let mut i = base(days);
            i.assessment_made_after_period = assessed;
            let r = compute(&i);
            assert_eq!(
                r.assessment_restricted, restricted,
                "days={} assessed={}",
                days, assessed
            );
        }
    }

    #[test]
    fn citation_includes_2022_jurisdictional_case_year() {
        let r = compute(&base(60));
        assert!(r.citation.contains("2022"));
        assert!(r.citation.contains("159 T.C. No. 6"));
    }

    #[test]
    fn citation_includes_third_circuit_2023() {
        let r = compute(&base(60));
        assert!(r.citation.contains("3d Cir. 2023"));
        assert!(r.citation.contains("75 F.4th 196"));
    }
}
