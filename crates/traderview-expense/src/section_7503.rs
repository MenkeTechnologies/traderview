//! IRC § 7503 — Time for performance of acts where last day
//! falls on Saturday, Sunday, or legal holiday. The
//! weekend/holiday extension rule that stacks with § 7502
//! timely-mailing rule + § 6213(a) Tax Court petition
//! window + § 6511 refund claim limitations. Trader-broadly-
//! applicable timing rule because every tax deadline
//! (return filing, payment, SNOD petition, refund claim,
//! election statement) is subject to § 7503's automatic
//! next-business-day extension when the deadline falls on
//! a non-business day. Companion to `section_7502`
//! (timely-mailing rule), `section_6213` (Tax Court
//! petition deadline), `section_6511` (refund limitations),
//! § 6072 (return due dates), § 6151 (payment due dates).
//!
//! **§ 7503 General rule** — when the last day prescribed
//! under authority of the internal revenue laws for
//! performing any act falls on **Saturday, Sunday, or a
//! legal holiday**, the performance of such act shall be
//! considered timely if it is performed on the **next
//! succeeding day which is not a Saturday, Sunday, or a
//! legal holiday**.
//!
//! **Authorized extensions included** — any authorized
//! extension of time shall be included in determining the
//! last day for performance of any act.
//!
//! **Legal holiday defined** — the term "legal holiday"
//! means:
//! 1. A **legal holiday in the District of Columbia**; AND
//! 2. In the case of any return, statement, or other
//!    document required to be filed, or any other act
//!    required under authority of the internal revenue
//!    laws to be performed, at any office of the Secretary
//!    or at any other office of the United States or any
//!    agency thereof, **located outside the District of
//!    Columbia but within an internal revenue district**,
//!    the term "legal holiday" ALSO means a **statewide
//!    legal holiday in the State where such office is
//!    located**.
//!
//! **Scope of application** — § 7503 applies to acts to be
//! performed by:
//! - the **taxpayer** (filing of any income, estate, or
//!   gift tax return; payment of any such tax; filing of
//!   § 6213(a) Tax Court petition; filing of § 6511 refund
//!   claim; filing of election statement); AND
//! - the **Commissioner**, district director, or director
//!   of a regional service center (issuance of § 6212
//!   SNOD; § 6303 notice and demand; § 6851 termination
//!   assessment notice).
//!
//! **DC legal holidays** (5 USC § 6103 + DC Code § 28-2701):
//! - New Year's Day (January 1)
//! - Martin Luther King Jr. Day (third Monday in January)
//! - Inauguration Day (January 20 every 4 years, DC only)
//! - Washington's Birthday (third Monday in February)
//! - Emancipation Day (April 16, DC only)
//! - Memorial Day (last Monday in May)
//! - Juneteenth National Independence Day (June 19, since
//!   2021)
//! - Independence Day (July 4)
//! - Labor Day (first Monday in September)
//! - Columbus Day / Indigenous Peoples Day (second Monday
//!   in October)
//! - Veterans Day (November 11)
//! - Thanksgiving Day (fourth Thursday in November)
//! - Christmas Day (December 25)
//!
//! **Emancipation Day quirk** — DC Emancipation Day (April
//! 16) regularly extends federal tax filing deadline by 1
//! business day when April 15 falls on a weekend or April
//! 16 itself falls on a weekend; this is why federal income
//! tax filing deadline is often April 17 or 18.
//!
//! Citations: 26 USC § 7503; 26 CFR § 301.7503-1; Rev. Rul.
//! 2015-13 (DC Emancipation Day); 5 USC § 6103 (federal
//! holidays); DC Code § 28-2701; § 7502 (timely-mailing
//! rule); § 6213(a) (Tax Court petition); § 6511 (refund
//! limitations); § 6072 (return due dates).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DayType {
    /// Regular business day (Monday-Friday, not a holiday).
    BusinessDay,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
    /// Federal legal holiday in DC (codified at 5 USC § 6103).
    FederalLegalHoliday,
    /// DC-only holiday (Emancipation Day April 16,
    /// Inauguration Day).
    DcOnlyHoliday,
    /// Statewide legal holiday in state where office
    /// located outside DC (§ 7503 second clause).
    StatewideLegalHolidayInOfficeState,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActScope {
    /// Act required at DC office (5 USC § 6103 holidays
    /// only).
    DcOfficeOnly,
    /// Act required at office outside DC but in internal
    /// revenue district (state holidays also count).
    OutsideDcInternalRevenueDistrict,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7503Input {
    pub last_day_type: DayType,
    pub act_scope: ActScope,
    /// Whether act was performed on the deadline day or on
    /// next non-weekend/non-holiday day.
    pub performed_on_or_before_next_business_day: bool,
    /// Days from deadline day to actual performance.
    pub days_from_deadline_to_performance: u32,
    /// Whether authorized extension was included in
    /// determining the last day.
    pub authorized_extension_included: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7503Result {
    pub extension_engaged: bool,
    pub deemed_timely: bool,
    pub last_day_is_weekend_or_holiday: bool,
    pub state_holiday_treatment_available: bool,
    pub authorized_extension_recognized: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7503Input) -> Section7503Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let is_weekend = matches!(input.last_day_type, DayType::Saturday | DayType::Sunday);

    let is_federal_holiday = matches!(
        input.last_day_type,
        DayType::FederalLegalHoliday | DayType::DcOnlyHoliday
    );

    let state_holiday_treatment_available =
        matches!(input.act_scope, ActScope::OutsideDcInternalRevenueDistrict);

    let is_state_holiday = matches!(
        input.last_day_type,
        DayType::StatewideLegalHolidayInOfficeState
    );

    let is_legal_holiday =
        is_federal_holiday || (is_state_holiday && state_holiday_treatment_available);

    let extension_engaged = is_weekend || is_legal_holiday;

    if is_state_holiday && !state_holiday_treatment_available {
        failure_reasons.push(
            "26 USC § 7503 — statewide legal holiday only counts when act is required at office OUTSIDE District of Columbia but within an internal revenue district; act at DC office uses only 5 USC § 6103 federal holidays".to_string(),
        );
    }

    let deemed_timely = if extension_engaged {
        input.performed_on_or_before_next_business_day
    } else {
        input.days_from_deadline_to_performance == 0
    };

    if extension_engaged && !input.performed_on_or_before_next_business_day {
        failure_reasons.push(
            "26 USC § 7503 — extension only saves performance on or before next succeeding day which is NOT Saturday, Sunday, or legal holiday".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 7503 — when last day for performing any act falls on Saturday, Sunday, or legal holiday, performance is timely if performed on next succeeding day which is not Saturday, Sunday, or legal holiday".to_string(),
        "26 USC § 7503 — any authorized extension of time shall be included in determining the last day for performance".to_string(),
        "26 USC § 7503 legal holiday defined: (1) legal holiday in District of Columbia AND (2) statewide legal holiday in State where office located outside DC but within internal revenue district".to_string(),
        "26 CFR § 301.7503-1 — § 7503 applies to acts by TAXPAYER (return filing + payment + § 6213 petition + § 6511 refund claim + elections) AND acts by COMMISSIONER (§ 6212 SNOD + § 6303 notice and demand + § 6851 termination notice)".to_string(),
        "Rev. Rul. 2015-13 — DC Emancipation Day (April 16) regularly extends federal tax filing deadline by 1 business day when April 15 falls on weekend or April 16 itself on weekend".to_string(),
        "5 USC § 6103 federal holidays (DC): New Year's Day + MLK Day + Washington's Birthday + Memorial Day + Juneteenth (since 2021) + Independence Day + Labor Day + Columbus Day + Veterans Day + Thanksgiving + Christmas Day".to_string(),
        "DC-only holidays: Emancipation Day (April 16) + Inauguration Day (January 20 every 4 years)".to_string(),
        "§ 7503 stacks with § 7502 timely-mailing rule — postmark on or before extended deadline counts as timely filing per § 7502(a) + § 7503 extension".to_string(),
        "Cross-references: § 7503 applies to § 6213(a) Tax Court petition window + § 6511 refund claim + § 6072 return due dates + § 6151 payment due dates + § 6212 SNOD issuance".to_string(),
    ];

    Section7503Result {
        extension_engaged,
        deemed_timely,
        last_day_is_weekend_or_holiday: extension_engaged,
        state_holiday_treatment_available,
        authorized_extension_recognized: input.authorized_extension_included,
        failure_reasons,
        citation: "26 USC § 7503; 26 CFR § 301.7503-1; Rev. Rul. 2015-13; 5 USC § 6103; DC Code § 28-2701; § 7502; § 6213(a); § 6511; § 6072",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section7503Input {
        Section7503Input {
            last_day_type: DayType::BusinessDay,
            act_scope: ActScope::DcOfficeOnly,
            performed_on_or_before_next_business_day: true,
            days_from_deadline_to_performance: 0,
            authorized_extension_included: false,
        }
    }

    #[test]
    fn business_day_deadline_no_extension() {
        let r = check(&valid_base());
        assert!(!r.extension_engaged);
        assert!(r.deemed_timely);
    }

    #[test]
    fn saturday_deadline_engages_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::Saturday;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(r.deemed_timely);
    }

    #[test]
    fn sunday_deadline_engages_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::Sunday;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(r.deemed_timely);
    }

    #[test]
    fn federal_legal_holiday_engages_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::FederalLegalHoliday;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(r.deemed_timely);
    }

    #[test]
    fn dc_only_holiday_engages_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::DcOnlyHoliday;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(r.deemed_timely);
    }

    #[test]
    fn state_holiday_at_outside_dc_office_engages_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::StatewideLegalHolidayInOfficeState;
        i.act_scope = ActScope::OutsideDcInternalRevenueDistrict;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(r.state_holiday_treatment_available);
    }

    #[test]
    fn state_holiday_at_dc_office_does_not_engage_extension() {
        let mut i = valid_base();
        i.last_day_type = DayType::StatewideLegalHolidayInOfficeState;
        i.act_scope = ActScope::DcOfficeOnly;
        let r = check(&i);
        assert!(!r.extension_engaged);
        assert!(!r.state_holiday_treatment_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("OUTSIDE District of Columbia")));
    }

    #[test]
    fn extension_engaged_but_not_performed_on_next_business_day_fails() {
        let mut i = valid_base();
        i.last_day_type = DayType::Saturday;
        i.performed_on_or_before_next_business_day = false;
        let r = check(&i);
        assert!(r.extension_engaged);
        assert!(!r.deemed_timely);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7503") && f.contains("next succeeding day")));
    }

    #[test]
    fn authorized_extension_recognized() {
        let mut i = valid_base();
        i.authorized_extension_included = true;
        let r = check(&i);
        assert!(r.authorized_extension_recognized);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 7503"));
        assert!(r.citation.contains("26 CFR § 301.7503-1"));
        assert!(r.citation.contains("Rev. Rul. 2015-13"));
        assert!(r.citation.contains("5 USC § 6103"));
        assert!(r.citation.contains("DC Code § 28-2701"));
        assert!(r.citation.contains("§ 7502"));
        assert!(r.citation.contains("§ 6213(a)"));
        assert!(r.citation.contains("§ 6511"));
        assert!(r.citation.contains("§ 6072"));
    }

    #[test]
    fn note_pins_general_rule_weekend_holiday_extension() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7503")
            && n.contains("Saturday, Sunday, or legal holiday")
            && n.contains("next succeeding day")));
    }

    #[test]
    fn note_pins_authorized_extension_inclusion() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("authorized extension of time") && n.contains("included")));
    }

    #[test]
    fn note_pins_legal_holiday_definition_two_clauses() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("(1) legal holiday in District of Columbia")
                && n.contains("(2) statewide legal holiday")));
    }

    #[test]
    fn note_pins_scope_taxpayer_and_commissioner_acts() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("TAXPAYER")
            && n.contains("COMMISSIONER")
            && n.contains("§ 6213")
            && n.contains("§ 6511")
            && n.contains("§ 6212")
            && n.contains("§ 6303")
            && n.contains("§ 6851")));
    }

    #[test]
    fn note_pins_rev_rul_2015_13_emancipation_day() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 2015-13")
            && n.contains("Emancipation Day")
            && n.contains("April 16")
            && n.contains("April 15")));
    }

    #[test]
    fn note_pins_5_usc_6103_federal_holiday_list() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("5 USC § 6103")
            && n.contains("Juneteenth")
            && n.contains("2021")
            && n.contains("Thanksgiving")
            && n.contains("MLK Day")));
    }

    #[test]
    fn note_pins_dc_only_holidays() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("DC-only")
            && n.contains("Emancipation Day")
            && n.contains("Inauguration Day")));
    }

    #[test]
    fn note_pins_7502_7503_stacking() {
        let r = check(&valid_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7503 stacks with § 7502") && n.contains("timely-mailing")));
    }

    #[test]
    fn note_pins_cross_references_collection_constellation() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6213(a)")
            && n.contains("§ 6511")
            && n.contains("§ 6072")
            && n.contains("§ 6151")
            && n.contains("§ 6212")));
    }

    #[test]
    fn day_type_truth_table_dc_office() {
        for (day, exp_engaged) in [
            (DayType::BusinessDay, false),
            (DayType::Saturday, true),
            (DayType::Sunday, true),
            (DayType::FederalLegalHoliday, true),
            (DayType::DcOnlyHoliday, true),
            (DayType::StatewideLegalHolidayInOfficeState, false),
        ] {
            let mut i = valid_base();
            i.last_day_type = day;
            i.act_scope = ActScope::DcOfficeOnly;
            let r = check(&i);
            assert_eq!(
                r.extension_engaged, exp_engaged,
                "day={:?} expected engaged={}",
                day, exp_engaged
            );
        }
    }

    #[test]
    fn day_type_truth_table_outside_dc_office() {
        for (day, exp_engaged) in [
            (DayType::BusinessDay, false),
            (DayType::Saturday, true),
            (DayType::Sunday, true),
            (DayType::FederalLegalHoliday, true),
            (DayType::DcOnlyHoliday, true),
            (DayType::StatewideLegalHolidayInOfficeState, true),
        ] {
            let mut i = valid_base();
            i.last_day_type = day;
            i.act_scope = ActScope::OutsideDcInternalRevenueDistrict;
            let r = check(&i);
            assert_eq!(
                r.extension_engaged, exp_engaged,
                "day={:?} expected engaged={}",
                day, exp_engaged
            );
        }
    }

    #[test]
    fn state_holiday_only_engages_outside_dc_invariant() {
        let mut i_dc = valid_base();
        i_dc.last_day_type = DayType::StatewideLegalHolidayInOfficeState;
        i_dc.act_scope = ActScope::DcOfficeOnly;
        let r_dc = check(&i_dc);
        assert!(!r_dc.extension_engaged);

        let mut i_outside = valid_base();
        i_outside.last_day_type = DayType::StatewideLegalHolidayInOfficeState;
        i_outside.act_scope = ActScope::OutsideDcInternalRevenueDistrict;
        let r_outside = check(&i_outside);
        assert!(r_outside.extension_engaged);
    }

    #[test]
    fn federal_holiday_engages_regardless_of_office_location_invariant() {
        for scope in [
            ActScope::DcOfficeOnly,
            ActScope::OutsideDcInternalRevenueDistrict,
        ] {
            let mut i = valid_base();
            i.last_day_type = DayType::FederalLegalHoliday;
            i.act_scope = scope;
            let r = check(&i);
            assert!(r.extension_engaged);
        }
    }
}
